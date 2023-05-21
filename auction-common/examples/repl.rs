use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use ed25519_dalek_fiat::{Keypair, Signer};
use rand::rngs;
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;
use serde::{Deserialize, Serialize};
use auction_common::auction::{Auction, AuctionSigned, Bid};
use thiserror::Error;
use crate::UseError::{AuctionEnded, InvalidId, InvalidValue};


type Result<T> = anyhow::Result<T>;

#[derive(Serialize, Deserialize)]
struct ReplContext {
    me: Option<Keypair>,
    house: Vec<AuctionSigned>
}

#[derive(Error, Debug)]
enum UseError {
    #[error("No data loaded")]
    NoData,
    #[error("Fail to Parse file")]
    DataLoadFailed,
    #[error("Not a valid Id")]
    InvalidId,
    #[error("Not a valid amount")]
    InvalidValue,
    #[error("Auction already ended")]
    AuctionEnded
}

impl ReplContext {
    pub fn new() -> Self {
        ReplContext {
            me: None,
            house: vec![]
        }
    }
}

fn init(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let mut rng = rngs::ThreadRng::default();
    let keypair = Keypair::generate(&mut rng);
    context.me = Some(keypair);
    Ok(None)
}

fn save(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    match &context.me {
        None => {
            Err(UseError::NoData)?
        }
        Some(_) => {
            let mut file = match args.get_one::<String>("file") {
                None => {
                    File::create("auction.data")?
                }
                Some(file) => {
                    File::create(file)?
                }
            };
            let bytes = bincode::serialize(context).unwrap();
            file.write_all(&bytes[..])?;
            Ok(None)
        }
    }
}

fn load(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let mut file = match args.get_one::<String>("file") {
        None => {
            File::open("auction.data")?
        }
        Some(file) => {
            File::open(file)?
        }
    };

    let mut res = Vec::new();
    file.read_to_end(&mut res)?;
    match bincode::deserialize(&res) {
        Ok(my_context) => {
            let _ = mem::replace(context, my_context);
            Ok(None)
        }
        Err(_) => {
            Err(UseError::DataLoadFailed)?
        }
    }
}

fn create_auction(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let name = args.get_one::<String>("name").unwrap().to_string();
    let product = args.get_one::<String>("product").unwrap().to_string();
    let minimum: u64 = args.get_one::<String>("minimum").unwrap().parse().unwrap();
    let auction = Auction::new(context.me.as_ref().unwrap().public, name.to_string(), product, minimum);
    let bytes = bincode::serialize(&auction).unwrap();
    context.house.push(auction.sign(context.me.as_ref().unwrap().sign(&bytes)));
    Ok(None)
}

fn ended_auction(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let auctions: Vec<_> = context.house.iter().filter(|&x| x.ended()).collect();
    list(auctions)
}

fn list_auction(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let auctions: Vec<_> = context.house.iter().filter(|&x| !x.ended()).collect();
    list(auctions)
}

fn list(list: Vec<&AuctionSigned>) -> Result<Option<String>> {
    let mut s = vec![];
    list.iter().for_each(|&x| s.push(x.to_string()));
    Ok(Some(s.join("\n").to_string()))
}

fn list_bids(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap().to_string();
    match hex::decode(id) {
        Ok(val) => {
            let auction = context.house.iter().filter(|x| x.get_id() == val);
            match auction.last() {
                None => {
                    Err(InvalidId)?
                }
                Some(auction) => {
                    let mut s = vec![];
                    auction.get_bids().iter().for_each(|x| s.push(x.to_string()));
                    Ok(Some(s.join("\n").to_string()))
                }
            }
        }
        Err(_) => {
            Err(InvalidId)?
        }
    }
}

fn bid_auction(args: ArgMatches, context: &mut ReplContext) -> Result<Option<String>> {
    let id = args.get_one::<String>("id").unwrap().to_string();
    let value: u64 = args.get_one::<String>("value").unwrap().parse().unwrap();
    match hex::decode(id) {
        Ok(val) => {
            let auction = context.house.iter_mut().filter(|x| x.get_id() == val);
            match auction.last() {
                None => {
                    Err(InvalidId)?
                }
                Some(auction_signed) => {
                    if auction_signed.ended() {
                        Err(AuctionEnded)?
                    }
                    if auction_signed.get_last_bid_or_minimum_bid() >= value {
                        Err(InvalidValue)?
                    }
                    let bid: Bid = Bid::new(value);
                    let bytes = bincode::serialize(&bid).unwrap();
                    let bid = bid.sign(context.me.as_ref().unwrap().public, context.me.as_ref().unwrap().sign(&bytes));
                    auction_signed.bid(bid);
                }
            }
            Ok(None)
        }
        Err(_) => {
           Err(InvalidId)?
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_env().unwrap();
    let mut repl = Repl::new(ReplContext::new(),
    ).with_command(
        Command::new("init")
            .about("Init the Auction house"),
        init
    ).with_command(
        Command::new("save")
            .arg(Arg::new("file").required(false))
            .about("Save Personal info to local file"),
        save
    ).with_command(
        Command::new("load")
            .arg(Arg::new("file").required(false))
            .about("Load Personal info from local file, WARNING: overwrites current values"),
        load
    ).with_command(
        Command::new("create")
            .arg(Arg::new("name").required(true))
            .arg(Arg::new("product").required(true))
            .arg(Arg::new("minimum").required(true))
            .about("Create a new auction"),
        create_auction
    ).with_command(
        Command::new("list")
            .about("List active auctions"),
        list_auction
    ).with_command(
        Command::new("ended")
            .about("List ended auctions"),
        ended_auction
    ).with_command(
        Command::new("bids")
            .arg(Arg::new("id").required(true).help("Auction ID"))
            .about("List all bids in auction"),
        list_bids
    ).with_command(
        Command::new("bid")
            .arg(Arg::new("id").required(true))
            .arg(Arg::new("value").required(true))
            .about("Bid in an auctions"),
        bid_auction
    );
    Ok(repl.run_async().await?)
}