use crate::{
    error::MaelstromError::{self, SerdeJsonError},
    message,
    message::{build_reply, WorkloadHandler},
    server::stdio::SharedIoServerContext,
};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Request = message::Request<RequestBody>;
pub type Response = message::Response<ResponseBody>;

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestBody {
    Init(Body),
}

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseBody {
    InitOk,
}

#[derive(Deserialize, Serialize, Constructor, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Body {
    node_id: String,
    node_ids: Vec<String>,
}

pub struct Handler;

impl WorkloadHandler for Handler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req = Request::new(serde_json::from_value(req)?);
        let _ = context.write().map(|mut c| {
            c.set_node(req.0.dest.clone());
            let RequestBody::Init(neighbors) = req.0.body.content.clone();
            c.set_neighbors(neighbors.node_ids.as_slice());
        });

        let response = build_reply(req, &context, ResponseBody::InitOk);
        serde_json::to_string(&response).map_err(SerdeJsonError)
    }
}
