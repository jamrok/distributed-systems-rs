use crate::{
    error::MaelstromError::{self, NoHandlerForRequestType, UnknownRequestType},
    message::{Message, RequestTypes},
    server::stdio::SharedIoServerContext,
};
use futures::future::{BoxFuture, FutureExt};
use serde_json::{from_value, Value};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct RouterLayer {
    context: SharedIoServerContext,
    handlers: Arc<HandlerMap>,
}

impl RouterLayer {
    pub fn new(context: SharedIoServerContext, handlers: Arc<HandlerMap>) -> Self {
        Self { context, handlers }
    }
}

impl<S> Layer<S> for RouterLayer
where
    S: Service<String>,
{
    type Service = RouterService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let context = self.context.clone(); // added step to increase code coverage
        let handlers = self.handlers.clone(); // added step to increase code coverage
        Self::Service {
            inner,
            context,
            handlers,
        }
    }
}

pub type HandlerFn = fn(SharedIoServerContext, Value) -> Result<String, MaelstromError>;
pub type HandlerMap = HashMap<RequestTypes, HandlerFn>;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct RouterService<S> {
    inner: S,
    context: SharedIoServerContext,
    handlers: Arc<HandlerMap>,
}

impl<S> Service<String> for RouterService<S>
where
    S: Service<String, Response = String, Error = MaelstromError>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: String) -> Self::Future {
        match self.route_message_to_handler(&req) {
            Ok(req) => self.inner.call(req).boxed(),
            Err(e) => self.inner.call(e.to_json_error(1).to_string()).boxed(),
        }
    }
}

impl<S> RouterService<S> {
    pub fn route_message_to_handler(&self, req: &str) -> Result<String, MaelstromError> {
        let req = Message::<Value>::from_str(req)?;

        let req_type = req
            .body
            .content
            .get("type")
            .ok_or(UnknownRequestType)?
            .clone();

        let req_type_enum = from_value::<RequestTypes>(req_type)?;

        let handler_fn = self
            .handlers
            .get(&req_type_enum)
            .ok_or(NoHandlerForRequestType)?;

        self.context
            .clone()
            .write()
            .map(|mut ctx| {
                if req_type_enum == RequestTypes::Topology {
                    dbg!(&req_type_enum, &req);
                } else {
                    dbg!(&req_type_enum);
                }
                ctx.message_type(req_type_enum);
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;

        handler_fn(self.context.clone(), req.to_value()?)
    }
}
