use crate::helper::{can_serde, test_with_registered_service};
use maelstrom_lib::{
    message::broadcast::{Request, Response},
    server::stdio::IoServerType,
};

pub const BROADCAST_REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "type": "broadcast",
            "msg_id": 42,
            "message": 1000
        }
    }
"#;

pub const BROADCAST_REQUEST_2: &str = r#"
    {
        "src": "c3",
        "dest": "n1",
        "body": {
            "type": "broadcast",
            "msg_id": 43,
            "message": 9001
        }
    }
"#;

const BROADCAST_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "type": "broadcast_ok",
            "msg_id": 2,
            "in_reply_to": 42
        }
    }
"#;

pub const READ_REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "type": "read",
            "msg_id": 44
        }
    }
"#;

const READ_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "in_reply_to": 44,
            "messages": [1000, 9001],
            "msg_id": 4,
            "type": "read_ok"
        }
    }
"#;

pub const SYNC_REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "msg_id": 42,
            "type": "sync",
            "messages": [1]
        }
    }
"#;

const SYNC_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "in_reply_to": 44,
            "messages": [1,1000, 9001],
            "msg_id": 4,
            "type": "read_ok"
        }
    }
"#;

pub const TOPOLOGY_REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "msg_id": 42,
            "type": "topology",
            "topology": {
                "n1": ["n2", "n3"],
                "n2": ["n1"],
                "n3": ["n1"]
            }
        }
    }
"#;

const TOPOLOGY_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "type": "topology_ok",
            "msg_id": 2,
            "in_reply_to": 42
        }
    }
"#;

#[tokio::test]
async fn broadcast_works_with_registered_service() {
    test_with_registered_service(
        vec![BROADCAST_REQUEST],
        BROADCAST_RESPONSE,
        IoServerType::Broadcast,
    )
    .await;
}

#[tokio::test]
async fn read_works_with_registered_service() {
    let input = vec![BROADCAST_REQUEST, BROADCAST_REQUEST_2, READ_REQUEST];
    test_with_registered_service(input, READ_RESPONSE, IoServerType::Broadcast).await;
}

#[tokio::test]
async fn sync_works_with_registered_service() {
    let input = vec![
        BROADCAST_REQUEST,
        BROADCAST_REQUEST_2,
        SYNC_REQUEST,
        READ_REQUEST,
    ];
    test_with_registered_service(input, SYNC_RESPONSE, IoServerType::Broadcast).await;
}

#[tokio::test]
async fn topology_works_with_registered_service() {
    test_with_registered_service(
        vec![TOPOLOGY_REQUEST],
        TOPOLOGY_RESPONSE,
        IoServerType::Broadcast,
    )
    .await;
}

#[tokio::test]
async fn test_serde_broadcast() {
    can_serde::<Request>(BROADCAST_REQUEST);
    can_serde::<Response>(BROADCAST_RESPONSE);
}

#[tokio::test]
async fn test_serde_read() {
    can_serde::<Request>(READ_REQUEST);
    can_serde::<Response>(READ_RESPONSE);
}

#[tokio::test]
async fn test_serde_sync() {
    can_serde::<Request>(SYNC_REQUEST);
}

#[tokio::test]
async fn test_serde_topology() {
    can_serde::<Request>(TOPOLOGY_REQUEST);
    can_serde::<Response>(TOPOLOGY_RESPONSE);
}
