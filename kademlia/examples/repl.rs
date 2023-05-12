use std::fs::{File};
use std::io::{Read, Write};
use std::mem;
use std::net::SocketAddr;
use std::str::FromStr;
use clap::{Arg, ArgMatches, Command};
use itertools::Itertools;
use reedline_repl_rs::{Repl};
use tokio::net::UdpSocket;
use dht::node::{LocalNode, LocalNodeBuilder};
use thiserror::Error;
use dht::kadid::KadID;

#[derive(Default)]
struct ReplContext {
    serial_data: Option<String>,
    builder: Option<LocalNodeBuilder>,
    node: Option<LocalNode>,
}

#[derive(Error, Debug)]
enum UseError {
    #[error("Invalid Argument")]
    InvalidArgument,
    #[error("No data loaded")]
    NoData,
    #[error("Node not built, nothing to start")]
    NoNodeBuilder,
    #[error("Node already exists")]
    AlreadyStarted,
    #[error("Node not running")]
    NotStarted,
    #[error("Incomplete command. Specify a subcommand to execute, then try again.")]
    IncompleteCommand
}


type Result<T> = anyhow::Result<T>;

fn load_file(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let mut file = File::open(args.get_one::<String>("file").unwrap())?;
    let mut res = String::new();
    file.read_to_string(&mut res)?;
    context.serial_data = Some(res);
    Ok(None)
}

async fn get_builder(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let addr = args.get_one::<String>("address").unwrap();
    let flag = args.get_one::<String>("from").unwrap();
    match flag.as_ref() {
        "empty" => {
            let builder = LocalNodeBuilder::start_empty(UdpSocket::bind(addr).await?);
            context.builder = Some(builder);
        }
        "data" => {
            if let Some(data) = &context.serial_data {
                let builder = LocalNodeBuilder::start_from_data(data, UdpSocket::bind(addr).await?)?;
                context.builder = Some(builder);
            } else {
                Err(UseError::NoData)?
            }
        }
        _ => Err(UseError::InvalidArgument)?,
    }

    Ok(None)
}

fn set_bootstrap(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &mut context.builder {
        None => Err(UseError::NoNodeBuilder)?,
        Some(builder) => {
            match args.subcommand() {
                None => Err(UseError::IncompleteCommand)?,
                Some((subc, args)) => {
                    match subc {
                        "none" => {
                            builder.bootstrap(false);
                        }
                        "addr" => {
                            let addr = args.get_one::<String>("addr").unwrap();
                            builder.bootstrap(true);
                            builder.bootstrap_addr(SocketAddr::from_str(addr)?);
                        }
                        &_ => unimplemented!()
                    }
                }
            }
        }
    }
    Ok(None)
}

async fn start_node(_args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    if context.node.is_some() {
        Err(UseError::AlreadyStarted)?
    }
    if let Some(builder) = mem::take(&mut context.builder) {
        let node = builder.build().await;
        context.node = Some(node);
    } else {
        Err(UseError::NoNodeBuilder)?
    }

    Ok(None)
}

async fn stop_node(_args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match mem::take(&mut context.node) {
        None => Err(UseError::NotStarted)?,
        Some(node) => {
            let data = node.destroy_serialized().await;
            context.serial_data = Some(data);
        }
    }

    Ok(None)
}

fn save_file(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &context.serial_data {
        None => {
            Err(UseError::NoData)?
        }
        Some(data) => {
            let mut file = File::create(args.get_one::<String>("file").unwrap())?;
            file.write_all(data.as_bytes())?
        }
    }
    Ok(None)
}

async fn node_lookup(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    if let Some(node) = &context.node {
        let key: KadID = args.get_one::<String>("key").unwrap().as_str().try_into()?;
        let res = node.lookup(key).await;
        Ok(Some(format!("{:?}", res)))
    } else {
        Err(UseError::NotStarted)?
    }
}

fn show_node_id(node: &LocalNode) -> Result<Option<String>> {
    Ok(Some(format!("{}",node.get_id())))
}

fn show_node_location(node: &LocalNode) -> Result<Option<String>> {
    Ok(Some(
        format!("Node bound to: {}", node.get_location()?)
    ))
}

fn show_buckets_subcommand(node: &LocalNode, args: &ArgMatches) -> Result<Option<String>> {
    let index = args.get_one::<String>("index").unwrap();
    match index.as_str() {
        "summary" => {
            Ok(Some(
                Itertools::intersperse(node.print_buckets_summary().into_iter(),"\n".parse()?).collect()
            ))
        }
        e => {
            let e: u8 = e.parse()?;
            match e {
                0..=159 => {
                    todo!()
                }
                _ => {
                    Err(UseError::InvalidArgument)?
                }
            }
        }
    }
}

fn show_node_storage(node: &LocalNode) -> Result<Option<String>> {
    Ok(Some(
        node.print_storage()
    ))
}

async fn show_node_info_dispatch(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &context.node {
        None => Err(UseError::NotStarted)?,
        Some(node) => {
            match args.subcommand() {
                None => Err(UseError::IncompleteCommand)?,
                Some((subc, args)) => {
                    match subc {
                        "id" => show_node_id(node),
                        "location" => show_node_location(node),
                        "buckets" => show_buckets_subcommand(node, args),
                        "storage" => show_node_storage(node),
                        &_ => unimplemented!()
                    }
                }
            }
        }
    }
}

async fn node_get(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &context.node {
        None => Err(UseError::NotStarted)?,
        Some(node) => {
            let key: KadID = args.get_one::<String>("key").unwrap().as_str().try_into()?;
            let value = node.get(key).await.map(|value| String::from_utf8_lossy(&value).into_owned());
            Ok(value)
        }
    }
}

async fn node_insert(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &context.node {
        None => Err(UseError::NotStarted)?,
        Some(node) => {
            let key: KadID = args.get_one::<String>("key").unwrap().as_str().try_into()?;
            let value: Vec<u8> =  args.get_one::<String>("value").unwrap().clone().into_bytes();
            node.store(key, value).await;
            Ok(None)
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut repl = Repl::new(ReplContext::default())
        .with_command(
            Command::new("load")
                .arg(Arg::new("file").required(true))
                .about("Load a file"),
            load_file
        )
        .with_command_async(
            Command::new("build")
                .arg(Arg::new("from").required(true))
                .arg(Arg::new("address").required(true))
                .about("Build a node. Does nothing until started"),
            |args, context| Box::pin(get_builder(args, context))
        )
        .with_command(
            Command::new("bootstrap")
                .subcommand(Command::new("none"))
                .subcommand(Command::new("addr")
                    .arg(Arg::new("addr").required(true)))
                .about("Setup bootstrap options"),
            set_bootstrap
        )
        .with_command_async(
            Command::new("start")
                .about("Start a node execution"),
            |args, context| Box::pin(start_node(args, context))
        )
        .with_command_async(
            Command::new("stop")
                .about("Stop a running node"),
            |args, context| Box::pin(stop_node(args, context))
        )
        .with_command(
            Command::new("save")
                .arg(Arg::new("file").required(true))
                .about("Save the currently loaded node data"),
            save_file
        )
        .with_command_async(
            Command::new("lookup")
                .arg(Arg::new("key").required(true))
                .about("Lookup the given node by key"),
            |args, context| Box::pin(node_lookup(args, context))
        )
        .with_command_async(
            Command::new("show")
                .subcommand(Command::new("id"))
                .subcommand(Command::new("location"))
                .subcommand(Command::new("buckets")
                    .arg(Arg::new("index").required(true)))
                .subcommand(Command::new("storage"))
                .about("Show information about a running node"),
            |args, context| Box::pin(show_node_info_dispatch(args,context))
        )
        .with_command_async(
            Command::new("get")
                .arg(Arg::new("key").required(true))
                .about("Get a value by key"),
            |args, context| Box::pin(node_get(args, context))
        )
        .with_command_async(
            Command::new("insert")
                .arg(Arg::new("key").required(true))
                .arg(Arg::new("value").required(true))
                .about("Store a value in the network"),
            |args, context| Box::pin(node_insert(args, context))
        );
    Ok(repl.run_async().await?)
}