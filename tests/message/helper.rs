use maelstrom_lib::{
    message::Message,
    server::stdio::{start_io_server, IoServerType},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str, to_string, Value};
use std::{fmt::Debug, time::Duration};

pub fn parse_json(json: &str) -> String {
    from_str::<Value>(json)
        .map_err(|e| format!("Error: {e}\nParsing: [{json}]"))
        .unwrap()
        .to_string()
}

pub fn process_output(output: Vec<u8>, expected: &str) -> String {
    let output = String::from_utf8(output).unwrap();
    dbg!(&output);
    let expected_output_value = from_str::<Message<Value>>(expected).unwrap().body.content;
    // TODO: Fix MultiMessageBug so we don't have to hack tests to do initialization
    let json: Vec<&str> = output.split('\n').collect::<Vec<&str>>();
    let json = &json
        .into_iter()
        .filter(|s| {
            if s.is_empty() || s == &"\n" {
                false
            } else {
                let output_value = from_str::<Message<Value>>(s).unwrap().body.content;
                // dbg!(&output_value.get("type"),);
                output_value.get("type") == expected_output_value.get("type")
            }
        })
        .collect::<String>();
    parse_json(json)
}

pub fn can_serde<T: DeserializeOwned + Serialize + Debug>(json: &str) {
    let json = parse_json(json);
    let ser: T = from_str::<T>(&json).expect("unable to convert from string");
    dbg!(&ser);
    let de = parse_json(&to_string(&ser).unwrap());
    dbg!(&de);
    assert_eq!(json, de);
}

pub fn serde_vec_to_string(request: Vec<&str>) -> String {
    request
        .into_iter()
        .map(parse_json)
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn insert_init(mut request: Vec<&str>) -> String {
    request.insert(0, crate::init::REQUEST);
    serde_vec_to_string(request)
    // TODO: Fix MultiMessageBug so we don't have to hack tests to do initialization
}

pub async fn test_with_registered_service(
    request: Vec<&str>,
    response: &str,
    io_type: IoServerType,
) {
    let input = &insert_init(request);
    dbg!(&input);
    let expected_output = &parse_json(response);
    let mut output = Vec::new();
    let _ = start_io_server(input.as_bytes(), &mut output, io_type).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let output = &process_output(output, expected_output);
    dbg!(&input, &output, response, expected_output,);
    assert_eq!(expected_output, output);
}
