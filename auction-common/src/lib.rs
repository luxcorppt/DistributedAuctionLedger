#![allow(unused)]

pub mod auction;

use std::cmp::{Ord, Ordering};
use serde::{Serialize, Deserialize};

use std::cmp::Ordering::{Less, Equal, Greater};
use std::time::SystemTime;
use ed25519_dalek_fiat::{PublicKey, Signature};
use log::LevelFilter::Off;
use time::OffsetDateTime;
use thiserror::Error;
use crate::TransactionError::SigningError;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Not signed or repeating signing")]
    SigningError
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Transaction {
    sender: PublicKey,
    receiver: PublicKey,
    timestamp: OffsetDateTime,
    amount: u64
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TransactionSemiSigned {
    transaction: Transaction,
    sender_signature: Signature
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TransactionSigned {
    transaction: Transaction,
    sender_signature: Signature,
    receiver_signature: Signature
}

impl Transaction {
    pub fn new(sender: PublicKey, receiver: PublicKey, amount: u64) -> Self {
        Transaction {
            sender,
            receiver,
            timestamp: OffsetDateTime::now_utc(),
            amount
        }
    }

    pub fn sign(self, signature: Signature) -> TransactionSemiSigned{
        TransactionSemiSigned {
            transaction: self,
            sender_signature: signature
        }
    }
}

impl TransactionSemiSigned {
    pub fn sign(self, signature: Signature) -> Result<TransactionSigned, TransactionError> {
        Ok(TransactionSigned {
            transaction: self.transaction,
            sender_signature: self.sender_signature,
            receiver_signature: signature
        })
    }
}

impl TransactionSigned {
    pub fn body_serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&self.transaction)
    }

    pub fn body_digest(&self) -> Result<String, postcard::Error> {
        Ok(sha256::digest(&*self.body_serialize()?))
    }
}

impl TransactionSemiSigned {
    pub fn body_serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&self.transaction)
    }

    pub fn body_digest(&self) -> Result<String, postcard::Error> {
        Ok(sha256::digest(&*self.body_serialize()?))
    }
}

impl Transaction {
    pub fn body_serialize(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&self)
    }

    pub fn body_digest(&self) -> Result<String, postcard::Error> {
        Ok(sha256::digest(&*self.body_serialize()?))
    }
}

impl PartialOrd<Self> for TransactionSigned {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.transaction.timestamp.partial_cmp(&other.transaction.timestamp)
    }
}

impl Ord for TransactionSigned {
    fn cmp(&self, other: &Self) -> Ordering {
        self.transaction.timestamp.cmp(&other.transaction.timestamp)
    }
}
