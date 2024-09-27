use assert_matches::assert_matches;
use maelstrom_lib::{
    error::MaelstromError::{self, SerdeJsonError},
    message::echo::Request,
};
use serde_json::json;

#[test]
fn parse_json() {
    let src = "c1";
    let dest = "n1";
    let msg_id = 1;
    let msg = "Please echo 35";
    let msg_type = "echo";

    let valid_json = json! ({
        "src": &src,
        "dest": &dest,
        "body": {
            "type": &msg_type,
            "msg_id": msg_id,
            "echo": &msg
        }
    });

    let invalid_json = json! ({
        "src": &src,
        "dest": &dest,
        "blah": {
            "type": &msg_type,
            "msg_id": msg_id,
            "echo": &msg
        }
    });

    // Deserialize from json and get message
    let actual_msg = serde_json::from_value::<Request>(valid_json).expect("Unable to parse json");
    dbg!(&actual_msg);

    // Deserialize from json and get error
    let actual_error: MaelstromError = serde_json::from_value::<Request>(invalid_json)
        .unwrap_err()
        .into();
    assert_matches!(actual_error, SerdeJsonError(..));
}
