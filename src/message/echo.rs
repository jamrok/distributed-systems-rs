use crate::{
    error::MaelstromError::{self, SerdeJsonError},
    message::{self, build_reply, WorkloadHandler},
    server::stdio::SharedIoServerContext,
};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

pub type Request = message::Request<RequestBody>;
pub type Response = message::Response<ResponseBody>;

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestBody {
    Echo(Body),
}

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseBody {
    EchoOk(Body),
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct Body {
    echo: Value,
}

pub struct Handler;

impl WorkloadHandler for Handler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req = Request::new(serde_json::from_value(req)?);
        let RequestBody::Echo(echo) = req.content().clone();

        let response = build_reply(req, &context, ResponseBody::EchoOk(echo));
        serde_json::to_string(&response).map_err(SerdeJsonError)
    }
}
