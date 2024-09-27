use crate::helper::{can_serde, test_with_registered_service};
use maelstrom_lib::{
    message::echo::{Request, Response},
    server::stdio::IoServerType,
};

pub const REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "type": "echo",
            "msg_id": 42,
            "echo": "Meaning of life"
        }
    }
"#;

const RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "type": "echo_ok",
            "echo": "Meaning of life",
            "msg_id": 2,
            "in_reply_to": 42
        }
    }
"#;

#[tokio::test]
async fn works_with_registered_service() {
    test_with_registered_service(vec![REQUEST], RESPONSE, IoServerType::Echo).await;
}

#[tokio::test]
async fn test_serde() {
    can_serde::<Request>(REQUEST);
    can_serde::<Response>(RESPONSE);
}
