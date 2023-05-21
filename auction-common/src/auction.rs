use ed25519_dalek_fiat::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use rand::{RngCore,rngs};
use time::{Duration, OffsetDateTime};
use hex::ToHex;
use log::LevelFilter::Off;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auction {
    id: Vec<u8>,
    name: String,
    owner: PublicKey,
    product: String,
    minimum: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionSigned {
    auction: Auction,
    signature: Signature,
    end_time: OffsetDateTime,
    bids: Vec<BidSigned>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bid {
    value: u64,
    timestamp: OffsetDateTime
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BidSigned {
    owner: PublicKey,
    bid: Bid,
    signature: Signature
}

impl Bid {
    pub fn new(value: u64) -> Self {
        Bid {
            value,
            timestamp: OffsetDateTime::now_utc()
        }
    }

    pub fn sign(self, owner: PublicKey, signature: Signature) -> BidSigned {
        BidSigned {
            owner,
            bid: self,
            signature
        }
    }
}

impl Auction {
    pub fn new(owner: PublicKey, name: String, product: String, minimum: u64) -> Self {
        Auction {
            id: utils::random(),
            owner,
            name,
            product,
            minimum
        }
    }

    pub fn sign(self, signature: Signature) -> AuctionSigned {
        AuctionSigned {
            auction: self,
            signature,
            end_time: OffsetDateTime::now_utc() + Duration::seconds(300),
            bids: vec![]
        }
    }
}

impl AuctionSigned {
    pub fn get_id(&self) -> &[u8] {
        &self.auction.id
    }
    pub fn get_name(&self) -> String {
        self.auction.name.to_string()
    }
    pub fn get_product(&self) -> String {
        self.auction.product.to_string()
    }
    pub fn get_minimum(&self) -> u64 {
        self.auction.minimum
    }

    pub fn get_last_bid_or_minimum_bid(&self) -> u64 {
        match self.bids.last() {
            None => {
                self.get_minimum()
            }
            Some(bid) => {
                bid.bid.value
            }
        }
    }

    pub fn bid(&mut self, bid: BidSigned){
        self.bids.push(bid);
        self.end_time = OffsetDateTime::now_utc() + Duration::seconds(300);
    }

    pub fn get_bids(&self) -> &Vec<BidSigned> {
        &self.bids
    }

    pub fn ended(&self) -> bool {
        OffsetDateTime::now_utc() > self.end_time
    }
}

impl ToString for AuctionSigned {
    fn to_string(&self) -> String {
        format!("Id: {} | Name: {} | Product: {} | Min: {} | Current: {} | timestamp: {} | Ended: {}",
               hex::encode(self.get_id()), self.get_name(), self.get_product(), self.get_minimum(), self.get_last_bid_or_minimum_bid(), self.end_time, self.ended())

    }
}

impl ToString for BidSigned {
    fn to_string(&self) -> String {
        format!("Owner: {} | {}", hex::encode(self.owner.to_bytes()), self.bid.to_string())

    }
}

impl ToString for Bid {
    fn to_string(&self) -> String {
        format!("Timestamp: {} | Value: {}", self.timestamp, self.value)

    }
}