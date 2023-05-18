use std::fs::{OpenOptions};
use std::io::{Read, Write};
use tokio::net::UdpSocket;
use dht::node::LocalNodeBuilder;
use clap::{Parser};

#[derive(Parser, Debug)]
struct Args {
    #[clap(default_value="./test.dht")]
    file_path: String,
    #[clap(default_value="127.0.0.1:9898")]
    local_address: String,
    #[clap(default_value_t=true)]
    bootstrap: bool
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&args.file_path)
        .unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    drop(file);

    let node = match LocalNodeBuilder::start_from_data(&data, UdpSocket::bind(&args.local_address).await.unwrap()) {
        Ok(n) => {n}
        Err(_) => {
            LocalNodeBuilder::start_empty(UdpSocket::bind(&args.local_address).await.unwrap())
        }
    }.bootstrap(false);

    let node = node.build().await;
    let data_out = node.destroy_serialized().await;

    let mut file = OpenOptions::new().truncate(true).write(true).open(&args.file_path).unwrap();
    file.write_all(data_out.as_bytes()).unwrap();
}
