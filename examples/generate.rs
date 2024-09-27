use maelstrom_lib::{
    error::MaelstromError,
    server::stdio::{start_io_server, IoServerType},
};
use std::io::{stdin, stdout};

#[tokio::main]
async fn main() -> Result<(), MaelstromError> {
    let input = stdin().lock();
    let output = stdout();
    start_io_server(input, output, IoServerType::Generate).await
}
