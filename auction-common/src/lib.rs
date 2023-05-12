#![allow(unused)]

use serde::{Serialize, Deserialize};

type UserPubKey = Vec<u8>;

#[derive(Serialize, Deserialize)]
struct TransactionEntry {
    user: UserPubKey,
    amount: u64,
}

#[derive(Serialize, Deserialize)]
struct TransactionSigning {
    sources: [TransactionEntry; 8],
    destinations: [TransactionEntry; 8],
}

type Signature = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    body: TransactionSigning,
    signatures: [Signature; 8]
}

impl Transaction {
    fn body_serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&self.body)
    }

    fn body_digest(&self) -> Result<String, postcard::Error> {
        Ok(sha256::digest(&*self.body_serialize()?))
    }
}
