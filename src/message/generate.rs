use crate::{
    error::MaelstromError::{self, SerdeJsonError},
    message,
    message::{build_reply, WorkloadHandler},
    server::stdio::SharedIoServerContext,
};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type Request = message::Request<RequestBody>;
pub type Response = message::Response<ResponseBody>;

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestBody {
    Generate,
}

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResponseBody {
    GenerateOk(Body),
}

#[derive(Deserialize, Serialize, Constructor, Clone, Debug, Eq, PartialEq)]
pub struct Body {
    id: Uuid,
}

pub struct Handler;

impl WorkloadHandler for Handler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req: Request = Request::new(serde_json::from_value(req)?);
        let id = Uuid::new_v4();
        let response = build_reply(req, &context, ResponseBody::GenerateOk(Body::new(id)));
        serde_json::to_string(&response).map_err(SerdeJsonError)
    }
}
