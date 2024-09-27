use crate::{
    bin_tests::IoServerType::{Broadcast, Echo, GCounter, Generate},
    helper::insert_init,
    init,
};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    io::Write,
    process::{Command, Stdio},
};

#[derive(Deserialize, Serialize, From, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IoServerType {
    Echo,
    Broadcast,
    GCounter,
    Generate,
}
impl Display for IoServerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self)
                .expect("Failed to serialize")
                .replace('"', "")
        )
    }
}
#[test]
fn test_binaries() {
    // Run the example using `cargo run --example`
    for bin in [Echo, Broadcast, GCounter, Generate] {
        let input = insert_init(Vec::from([init::REQUEST])).into_bytes();
        let mut output = Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg(bin.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute binary");

        let stdin = output.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(input.as_slice())
            .expect("Failed to write to stdin");

        let output = output.wait_with_output().expect("No go");
        let stdout = String::from_utf8(output.stdout).expect("No STDOUT Access");
        let stderr = String::from_utf8(output.stderr).expect("No STDERR Access");

        // Ensure response is as expected
        assert!(stdout.contains("init_ok"));

        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
    }
}
