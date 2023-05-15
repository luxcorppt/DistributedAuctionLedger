#![allow(unused)]

use std::cmp::{Ord, Ordering};
use serde::{Serialize, Deserialize};

use compare::{Compare};
use std::cmp::Ordering::{Less, Equal, Greater};
use std::time::SystemTime;

type UserPubKey = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct TransactionEntry {
    user: UserPubKey,
    amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct TransactionSigning {
    sources: [TransactionEntry; 8],
    destinations: [TransactionEntry; 8],
}

type Signature = Vec<u8>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, Eq, PartialEq)]
pub struct Transaction {
    body: TransactionSigning,
    timestamp: u128,
    signatures: [Signature; 8]
}

impl Transaction {
    pub fn new(amount: u64) -> Self {
        let entry00 = TransactionEntry { user: vec![], amount };
        let entry01 = TransactionEntry { user: vec![], amount };
        let entry02 = TransactionEntry { user: vec![], amount };
        let entry03 = TransactionEntry { user: vec![], amount };
        let entry04 = TransactionEntry { user: vec![], amount };
        let entry05 = TransactionEntry { user: vec![], amount };
        let entry06 = TransactionEntry { user: vec![], amount };
        let entry07 = TransactionEntry { user: vec![], amount };

        let entry10 = TransactionEntry { user: vec![], amount };
        let entry11 = TransactionEntry { user: vec![], amount };
        let entry12 = TransactionEntry { user: vec![], amount };
        let entry13 = TransactionEntry { user: vec![], amount };
        let entry14 = TransactionEntry { user: vec![], amount };
        let entry15 = TransactionEntry { user: vec![], amount };
        let entry16 = TransactionEntry { user: vec![], amount };
        let entry17 = TransactionEntry { user: vec![], amount };

        let sig0 = vec![0 as u8; 32] as Signature;
        let sig1 = vec![0 as u8; 32] as Signature;
        let sig2 = vec![0 as u8; 32] as Signature;
        let sig3 = vec![0 as u8; 32] as Signature;
        let sig4 = vec![0 as u8; 32] as Signature;
        let sig5 = vec![0 as u8; 32] as Signature;
        let sig6 = vec![0 as u8; 32] as Signature;
        let sig7 = vec![0 as u8; 32] as Signature;

        Transaction {
            body: TransactionSigning {
                sources: [entry00, entry01, entry02, entry03, entry04, entry05, entry06, entry07],
                destinations: [entry10, entry11, entry12, entry13, entry14, entry15, entry16, entry17],
            },
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
            signatures: [sig0, sig1, sig2, sig3, sig4, sig5, sig6, sig7],
        }
    }

    fn body_serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&self.body)
    }

    fn body_digest(&self) -> Result<String, postcard::Error> {
        Ok(sha256::digest(&*self.body_serialize()?))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        compare::natural().compare(&self.timestamp, &other.timestamp)
    }
}