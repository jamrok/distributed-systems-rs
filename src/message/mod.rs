use crate::{error::MaelstromError, server::stdio::SharedIoServerContext};
use derive_more::{Constructor, From};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Error, Value};
use std::str::FromStr;

pub mod broadcast;
pub mod echo;
pub mod g_counter;
pub mod generate;
pub mod init;

pub type MsgId = u64;

#[derive(Deserialize, Serialize, Constructor, From, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Message<T: Serialize> {
    pub src: String,
    pub dest: String,
    pub body: Body<T>,
}

#[derive(Deserialize, Serialize, Constructor, From, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Body<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<MsgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<MsgId>,
    #[serde(flatten)]
    pub content: T,
}

#[derive(Deserialize, Serialize, Constructor, From, Clone, Debug, Eq, PartialEq)]
pub struct Request<T: Serialize>(Message<T>);

#[derive(Deserialize, Serialize, Constructor, From, Clone, Debug, Eq, PartialEq)]
pub struct Response<T: Serialize>(Message<T>);

impl<T: Serialize> Request<T> {
    pub fn content(&self) -> &T {
        &self.0.body.content
    }

    pub fn serde_to_string(&self) -> Result<String, MaelstromError> {
        serde_json::to_string(&self).map_err(MaelstromError::SerdeJsonError)
    }
}

impl<T: Serialize> Response<T> {
    pub fn content(mut self, content: T) {
        self.0.body.content = content;
    }

    fn serde_to_string(&self) -> Result<String, MaelstromError> {
        serde_json::to_string(&self).map_err(MaelstromError::SerdeJsonError)
    }
}

impl<T: Serialize + DeserializeOwned> FromStr for Message<T> {
    type Err = MaelstromError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(MaelstromError::SerdeJsonError)
    }
}

fn build_reply<T: Serialize, R: Serialize>(
    req: Request<T>,
    ctx: &SharedIoServerContext,
    content: R,
) -> Response<R> {
    let mut ctx = ctx.write().unwrap();
    let node_id = ctx.node().to_string();
    let msg_id = ctx.next_msg_id();
    let dest = if req.0.src == node_id {
        req.0.dest
    } else {
        req.0.src
    };

    Response(Message {
        src: node_id,
        dest,
        body: Body {
            msg_id: Some(msg_id),
            in_reply_to: req.0.body.msg_id,
            content,
        },
    })
}

/// Sends a custom workload request.
///
/// # Panics
///
/// Might panic if the write lock is already held by the current thread.
pub fn send_request<T: Serialize>(
    dest: String,
    ctx: &SharedIoServerContext,
    content: T,
) -> Request<T> {
    let mut ctx = ctx
        .write()
        .expect("Unable to write to STDOUT (lock failed)");
    let src = ctx.node().to_string();
    let msg_id = ctx.next_msg_id();
    Request(Message {
        src,
        dest,
        body: Body {
            msg_id: Some(msg_id),
            in_reply_to: None,
            content,
        },
    })
}

impl<T: Serialize + DeserializeOwned> Message<T> {
    pub fn to_value(self) -> Result<Value, Error> {
        serde_json::to_value(self)
    }
}

impl<Req, Resp> From<(Message<Req>, Resp)> for Message<Resp>
where
    Req: Serialize,
    Resp: Serialize + DeserializeOwned,
{
    fn from(value: (Message<Req>, Resp)) -> Self {
        let (value, body) = value;
        Self {
            src: value.dest,
            dest: value.src,
            body: Body {
                msg_id: value.body.msg_id,
                in_reply_to: value.body.msg_id,
                content: body,
            },
        }
    }
}

pub trait WorkloadHandler {
    fn response(context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError>;
}

#[derive(Deserialize, Serialize, Default, Debug, From, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RequestTypes {
    #[default]
    Add,
    Init,
    Echo,
    Generate,
    Broadcast,
    Read,
    Topology,
    Sync,
    SyncCounter,
    SyncOk,
}
