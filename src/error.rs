use crate::message::MsgId;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::string::FromUtf8Error;
use thiserror;

pub type ErrCode = u16;
#[allow(clippy::module_name_repetitions)]
#[derive(thiserror::Error, Debug)]
pub enum MaelstromError {
    #[error("End of input")]
    EndOfInput,

    #[error("JoinError: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    /// The first message the node received was valid, but it wasn't the initialization message.
    /// The client's request did not conform to the server's expectations,
    /// and could not possibly have been processed.
    #[error("Invalid client request: {0}")]
    MalformedRequest(serde_json::Error),

    #[error("No message ID found in request body")]
    MissingMessageId,

    #[error("No workload handlers registered")]
    MissingWorkloadHandlers,

    /// The node received another Initialization message
    #[error("Node is already initialized.")]
    NodeAlreadyInitialized,

    /// The first message the node received was valid, but it wasn't the initialization message.
    #[error("Node got a valid message, but it was not the 'init' message.")]
    NodeNotInitialized,

    #[error("Context poison error: {0}")]
    PoisonError(String),

    #[error("Context RW lock error: {0}")]
    RWLockError(String),

    #[error("Serde Json Error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Error reading from STDIN")]
    StdinReadError(#[from] std::io::Error),

    #[error("Error reading Utf8 from STDIN")]
    StdinUtf8ReadError(#[from] FromUtf8Error),

    #[error("No handler found for request type")]
    NoHandlerForRequestType,

    #[error("Unknown request type received")]
    UnknownRequestType,

    /// Indicates that some kind of general, indefinite error occurred.
    /// Use this as a catch-all for errors you can't otherwise categorize,
    /// or as a starting point for your error handler:
    /// it's safe to return `internal-error` for every problem by default,
    /// then add special cases for more specific errors later.
    // Ref: https://github.com/jepsen-io/maelstrom/blob/8b9e94c75e59250b82d1730d923f9f8e088ee227/doc/protocol.md?#errors
    #[error("Unexpected error occurred")]
    Crash,
}

impl MaelstromError {
    fn code(&self) -> ErrCode {
        match self {
            MaelstromError::EndOfInput => 1000,
            MaelstromError::JoinError(_) => 1013,
            MaelstromError::MalformedRequest(_) => 1001,
            MaelstromError::MissingWorkloadHandlers => 1002,
            MaelstromError::NodeAlreadyInitialized => 1003,
            MaelstromError::NodeNotInitialized => 1004,
            MaelstromError::SerdeJsonError(_) => 1005,
            MaelstromError::StdinReadError(_) => 1006,
            MaelstromError::StdinUtf8ReadError(_) => 1007,
            MaelstromError::NoHandlerForRequestType => 1008,
            MaelstromError::UnknownRequestType => 1009,
            MaelstromError::Crash => 13,
            MaelstromError::MissingMessageId => 1010,
            MaelstromError::RWLockError(_) => 1011,
            MaelstromError::PoisonError(_) => 1012,
        }
    }

    #[must_use]
    pub fn to_json_error(&self, msg_id: MsgId) -> Value {
        let e_type = "error";
        let unexpected_error = json!(
            {
            "type": e_type,
            "in_reply_to": msg_id,
            "code": Self::Crash.code(),
            "text": Self::Crash.to_string(),
            }
        );
        let body = MaelstromErrorBody::new(e_type.into(), msg_id, self.code(), self.to_string());
        serde_json::to_value(body).unwrap_or(unexpected_error)
    }
}

#[derive(Serialize, Deserialize, Constructor, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MaelstromErrorBody {
    #[serde(rename = "type")]
    e_type: String,
    in_reply_to: MsgId,
    code: ErrCode,
    text: String,
}
