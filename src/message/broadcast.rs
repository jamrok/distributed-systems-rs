use crate::{
    error::MaelstromError::{self, SerdeJsonError},
    message::{self, build_reply, WorkloadHandler},
    server::stdio::{BroadcastContext, NumericMessage, SharedIoServerContext},
};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

pub type Request = message::Request<RequestBody>;
pub type Response = message::Response<ResponseBody>;

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestBody {
    Broadcast(Body),
    Read,
    Topology(TopologyBody),
    Sync(SyncBody),
}

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseBody {
    BroadcastOk,
    ReadOk(ReadOkBody),
    TopologyOk,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct Body {
    message: NumericMessage,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct SyncBody {
    messages: Vec<NumericMessage>,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct MessageBody {
    messages: String,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct TopologyBody {
    topology: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct ReadOkBody {
    messages: Vec<NumericMessage>,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct SyncOkBody {
    messages: Vec<NumericMessage>,
}

pub struct Handler;

impl WorkloadHandler for Handler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req = Request::new(serde_json::from_value(req)?);
        let ctx = &context;
        let source = req.0.src.clone();
        let body = match req.0.body.content.clone() {
            RequestBody::Broadcast(body) => Self::process_broadcast(&context, source, &body),
            RequestBody::Read => Self::process_read(&context),
            RequestBody::Topology(body) => Self::process_topology(&context, &body),
            RequestBody::Sync(body) => {
                let _ = Self::process_sync(&context, source, body.messages);
                return Ok(String::new());
            }
        }?;

        let response = build_reply(req, ctx, body);
        serde_json::to_string(&response).map_err(SerdeJsonError)
    }
}

impl Handler {
    pub fn process_broadcast(
        context: &SharedIoServerContext,
        source: String,
        body: &Body,
    ) -> Result<ResponseBody, MaelstromError> {
        context
            .write()
            .map(|mut ctx| ctx.add_message(source, body.message))
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(ResponseBody::BroadcastOk)
    }
    pub fn process_read(context: &SharedIoServerContext) -> Result<ResponseBody, MaelstromError> {
        let response = context
            .write()
            .map(|ctx| {
                let messages = ctx.clone().messages();
                ResponseBody::ReadOk(ReadOkBody { messages })
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(response)
    }

    pub fn process_sync(
        context: &SharedIoServerContext,
        source: String,
        messages: Vec<NumericMessage>,
    ) -> Result<ResponseBody, MaelstromError> {
        let response = context
            .write()
            .map(|mut ctx| {
                let _queued = ctx.synced(source, HashSet::from_iter(messages));
                ResponseBody::BroadcastOk
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(response)
    }

    pub fn process_topology(
        context: &SharedIoServerContext,
        body: &TopologyBody,
    ) -> Result<ResponseBody, MaelstromError> {
        context
            .write()
            .map(|mut ctx| {
                let node_id = ctx.node();
                let topology = body.topology.get(node_id);
                if let Some(nodes) = topology {
                    ctx.set_neighbors(nodes);
                };
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(ResponseBody::TopologyOk)
    }
}
