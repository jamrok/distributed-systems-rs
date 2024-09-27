use crate::{
    error::MaelstromError::{self},
    message::{self, build_reply, WorkloadHandler},
    server::stdio::{GCounterContext, NodeCounters, NumericMessage, SharedIoServerContext},
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
    Add(AddBody),
    Read,
    SyncCounter(SyncBody),
}

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseBody {
    AddOk,
    ReadOk(ReadOkBody),
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct AddBody {
    delta: NumericMessage,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct SyncBody {
    messages: NodeCounters,
}

#[derive(Deserialize, Serialize, From, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct ReadOkBody {
    value: NumericMessage,
}

pub struct Handler;

impl WorkloadHandler for Handler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req = Request::new(serde_json::from_value(req)?);
        let ctx = context.clone();
        let body = match req.0.body.content.clone() {
            RequestBody::Add(body) => Self::process_add(&context, &body),
            RequestBody::Read => Self::process_read(&context),
            RequestBody::SyncCounter(body) => return Self::process_sync(&context, body.messages),
        }?;

        build_reply(req, &ctx, body).serde_to_string()
    }
}

impl Handler {
    pub fn process_add(
        context: &SharedIoServerContext,
        body: &AddBody,
    ) -> Result<ResponseBody, MaelstromError> {
        context
            .write()
            .map(|mut ctx| {
                ctx.update_local_node_counter(body.delta);
                let node = ctx.node().to_string();
                if body.delta > 0 {
                    ctx.add_node_counter(node, body.delta);
                }
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;

        Ok(ResponseBody::AddOk)
    }

    pub fn process_read(context: &SharedIoServerContext) -> Result<ResponseBody, MaelstromError> {
        let response = context
            .read()
            .map(|ctx| {
                let value = ctx.counter();
                ResponseBody::ReadOk(ReadOkBody { value })
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(response)
    }

    pub fn process_sync(
        context: &SharedIoServerContext,
        new_counters: NodeCounters,
    ) -> Result<String, MaelstromError> {
        context
            .write()
            .map(|mut ctx| {
                ctx.update_all_node_counters(new_counters);
                let max_neighbor_counter = ctx.max_node_counter();
                let _ = ctx.cas_local_node_counter(max_neighbor_counter);
            })
            .map_err(|e| MaelstromError::PoisonError(e.to_string()))?;
        Ok(String::new())
    }
}
