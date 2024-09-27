use crate::helper::{can_serde, insert_init, process_output};
use maelstrom_lib::{
    message::generate::{Request, Response},
    server::stdio::{start_io_server, IoServerType},
};
use serde_json::{from_str, Value};
use std::str::FromStr;
use uuid::Uuid;

pub const REQUEST: &str = r#"
    {
        "src": "c5",
        "dest": "n1",
        "body": {
            "type": "generate",
            "msg_id": 42
        }
    }
"#;

const RESPONSE: &str = r#"
    {
        "src": "n1",
        "dest": "c5",
        "body": {
            "type": "generate_ok",
            "id": "a8fefd10-fb07-4b1a-bd66-96b6ae354b0c",
            "in_reply_to": 42,
            "msg_id": 2
        }
    }
"#;

#[tokio::test]
async fn works_with_registered_service() {
    let input = &insert_init(vec![REQUEST]);
    let mut output = Vec::new();
    let _ = start_io_server(input.as_bytes(), &mut output, IoServerType::Generate).await;

    let expected_output = from_str::<Value>(RESPONSE).unwrap();
    let output = &process_output(output, RESPONSE);
    let mut actual_output = from_str::<Value>(output).unwrap();

    // Get the UUID from the expected output and set it to the match the actual output
    let expected_uuid = &expected_output["body"]["id"];
    let actual_uuid = &actual_output.clone()["body"]["id"];
    let actual_uuid = actual_uuid.as_str().unwrap();
    actual_output["body"]["id"] = expected_uuid.clone();

    dbg!(&actual_output, &expected_uuid, actual_uuid);
    // Ensure UUID was generated in actual output
    Uuid::from_str(actual_uuid).expect("UUID not found in actual output");
    dbg!(&input, &output, RESPONSE, &expected_output);
    assert_eq!(expected_output, actual_output);
}

#[tokio::test]
async fn test_serde() {
    can_serde::<Request>(REQUEST);
    can_serde::<Response>(RESPONSE);
}
