use crate::helper::{can_serde, parse_json};
use maelstrom_lib::{
    error::MaelstromError::{self, SerdeJsonError},
    message::{Body, Message, RequestTypes::Echo, WorkloadHandler},
    server::stdio::{send_message, IoServer, SharedIoServerContext},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Request(Message<Value>);

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Response(Message<Value>);

pub struct MockEchoHandler {}

impl WorkloadHandler for MockEchoHandler {
    fn response(_context: SharedIoServerContext, req: Value) -> Result<String, MaelstromError> {
        let req: Request = serde_json::from_value(req)?;
        let Request(Message { src, dest, body }) = req;

        let response = Response(Message {
            src: dest,
            dest: src,
            body: Body {
                msg_id: None,
                in_reply_to: None,
                content: json!({
                    "type": "echo_ok",
                    "msg": body.content.get("msg").unwrap()
                }),
            },
        });
        serde_json::to_string(&response).map_err(SerdeJsonError)
    }
}

pub const REQUEST: &str = r#"
    {
        "src": "c1",
        "dest": "n1",
        "body": {
            "type": "echo",
            "msg": "The answer is 42"
        }
    }
"#;

const RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c1",
        "body": {
            "type": "echo_ok",
            "msg": "The answer is 42"
        }
    }
"#;

#[tokio::test]
async fn works_with_registered_service2() {
    let input = &parse_json(REQUEST);
    let expected_output = &parse_json(RESPONSE);
    let mut output = Vec::new();
    let _ = IoServer::new(input.as_bytes(), &mut output)
        .register(Echo, MockEchoHandler::response)
        .serve()
        .await;
    let output = parse_json(&String::from_utf8(output).unwrap());
    dbg!(&input, &output, RESPONSE, expected_output);
    assert_eq!(expected_output, &output);
}

#[tokio::test]
async fn test_serde() {
    can_serde::<Request>(REQUEST);
    can_serde::<Response>(RESPONSE);
}

#[tokio::test]
async fn fails_if_not_registered_with_service() {
    let input = &parse_json(REQUEST);
    let mut output = Vec::new();
    let _ = IoServer::new(input.as_bytes(), &mut output).serve().await;
    let output = String::from_utf8(output).unwrap();
    dbg!(&input, &output);
    assert!(output.contains("No handler found for request type"));
}

#[tokio::test]
async fn test_send_message() {
    let buffer = Vec::new();
    let expected = "Test Message".to_string();
    let actual = send_message(buffer, expected.clone())
        .await
        .expect("send_message should return data");
    assert_eq!(expected, actual);
}
