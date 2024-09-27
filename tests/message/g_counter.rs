use crate::helper::{can_serde, test_with_registered_service};
use maelstrom_lib::{
    message::g_counter::{Request, Response},
    server::stdio::IoServerType,
};

pub const ADD_REQUEST: &str = r#"
    {
        "src": "c2",
        "dest": "n1",
        "body": {
            "type": "add",
            "msg_id": 1,
            "delta": 40
        }
    }
"#;

pub const ADD_REQUEST_2: &str = r#"
    {
        "src": "c3",
        "dest": "n1",
        "body": {
            "type": "add",
            "msg_id": 3,
            "delta": 2
        }
    }
"#;

const ADD_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c2",
        "body": {
            "type": "add_ok",
            "in_reply_to": 1,
            "msg_id": 2
        }
    }
"#;

pub const READ_REQUEST: &str = r#"
    {
        "src": "c1",
        "dest": "n1",
        "body": {
            "type": "read",
            "msg_id": 14
        }
    }
"#;

const READ_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c1",
        "body": {
            "in_reply_to": 14,
            "value": 42,
            "msg_id": 4,
            "type": "read_ok"
        }
    }
"#;

pub const SYNC_REQUEST: &str = r#"
    {
        "src": "n1",
        "dest": "n2",
        "body": {
            "msg_id": 4,
            "type": "sync_counter",
            "messages": {"n1": 42}
        }
    }
"#;

const SYNC_RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c1",
        "body": {
            "in_reply_to": 14,
            "msg_id": 4,
            "value": 42,
            "type": "read_ok"
        }
    }
"#;

#[tokio::test]
async fn add_works_with_registered_service() {
    test_with_registered_service(vec![ADD_REQUEST], ADD_RESPONSE, IoServerType::Gcounter).await;
}

#[tokio::test]
async fn read_works_with_registered_service() {
    let input = vec![ADD_REQUEST, ADD_REQUEST_2, READ_REQUEST];
    test_with_registered_service(input, READ_RESPONSE, IoServerType::Gcounter).await;
}

#[tokio::test]
async fn sync_works_with_registered_service() {
    let input = vec![ADD_REQUEST, ADD_REQUEST_2, SYNC_REQUEST, READ_REQUEST];
    test_with_registered_service(input, SYNC_RESPONSE, IoServerType::Gcounter).await;
}

#[tokio::test]
async fn test_serde_gcounter() {
    can_serde::<Request>(ADD_REQUEST);
    can_serde::<Response>(ADD_RESPONSE);
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
