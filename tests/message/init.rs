use crate::helper::{can_serde, test_with_registered_service};
use maelstrom_lib::{
    message::init::{Request, Response},
    server::stdio::IoServerType,
};

pub const REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "type": "init",
            "msg_id": 4,
            "node_id": "n0",
            "node_ids": [ "c1", "c2", "c3" ]
        }
    }
"#;

const RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "msg_id": 1,
            "type": "init_ok",
            "in_reply_to": 4
        }
    }
"#;

#[tokio::test]
async fn works_with_registered_service() {
    test_with_registered_service(vec![], RESPONSE, IoServerType::Init).await;
}

#[tokio::test]
async fn test_serde() {
    can_serde::<Request>(REQUEST);
    can_serde::<Response>(RESPONSE);
}
