use std::ffi::c_long;
use sha1::Digest;
use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use enum_dispatch::enum_dispatch;
use auction_common::Transaction;

type MyHash = Vec<u8>;

// [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
//                              0
//              1                               2
//      3               4               5               6
//  7       8       9       10      11      12      13      14
//
#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleTree {
    tree: Vec<MyHash>
}


impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            tree: Vec::new()
        }
    }

    pub fn from_transactions(mut transactions: Vec<Transaction>) -> Self {
        let levels = (transactions.len() as f32).log2().ceil();
        transactions.sort();
        println!("{:?}", transactions);
        println!("Blocks: {} | Levels: {}", transactions.len(), levels);
        MerkleTree {
            tree: Vec::new()
        }
    }
}