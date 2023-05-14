use std::cmp::Ordering;
use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use auction_common::Transaction;
use crate::chain::{Chain};
use compare::{Compare};
use std::cmp::Ordering::{Less, Equal, Greater};

pub enum BlockError {
    DeserializeError,
    InvalidBlock
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block{
    index: u32,
    timestamp: u128,
    prev_hash: Vec<u8>,
    nonce: u64,
    transactions: Vec<Transaction>,
    difficulty: u64
}

impl Block {
    pub fn new(index: u32, timestamp: u128, prev_hash: Vec<u8>, transactions: Vec<Transaction>, difficulty: u64) -> Self {
        Block {
            index, timestamp, prev_hash, nonce: 0, transactions, difficulty
        }
    }

    pub fn complete_pow(mut self) -> BlockCompletePoW {
        loop {
            let bytes_block = bincode::serialize(&self).unwrap();
            let digest = sha1::Sha1::digest(&bytes_block);
            if verify_block_difficulty(&self, &digest[..]) {
                return BlockCompletePoW {
                    hash: digest.to_vec(),
                    block_inner: self
                }
            }
            self.nonce += 1
        }
    }

    pub fn complete_pos(self, signing_id: Vec<u8>, signature: Vec<u8>) -> BlockCompletePoS {
        BlockCompletePoS {
            signing_id,
            signature,
            block_inner: self
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BlockComplete {
    POW(BlockCompletePoW),
    POS(BlockCompletePoS)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockCompletePoW {
    hash: Vec<u8>,
    block_inner: Block
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockCompletePoS {
    signature: Vec<u8>,
    signing_id: Vec<u8>, // TODO: Change type to KadID
    block_inner: Block
}

impl BlockComplete {
    pub fn block_hash(&self) -> &[u8] {
        match self {
            BlockComplete::POW(block) => {
                block.block_hash()
            }
            BlockComplete::POS(block) => {
                block.block_hash()
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            BlockComplete::POW(block) => {
                block.is_valid()
            }
            BlockComplete::POS(block) => {
                block.is_valid()
            }
        }
    }

    pub fn get_prev_hash(&self) -> &Vec<u8>  {
        match self {
            BlockComplete::POW(block) => {
                block.get_prev_hash()
            }
            BlockComplete::POS(block) => {
                block.get_prev_hash()
            }
        }
    }

    pub fn get_index(&self) -> &u32  {
        match self {
            BlockComplete::POW(block) => {
                block.get_index()
            }
            BlockComplete::POS(block) => {
                block.get_index()
            }
        }
    }

    pub fn get_difficulty(&self) -> &u64  {
        match self {
            BlockComplete::POW(block) => {
                block.get_difficulty()
            }
            BlockComplete::POS(block) => {
                block.get_difficulty()
            }
        }
    }
}

impl BlockCompletePoW {
    pub fn block_hash(&self) -> &[u8] {
        &self.hash[..]
    }

    fn is_valid(&self) -> bool {
        let bytes = bincode::serialize(&self.block_inner).unwrap();
        let digest_trusted = sha1::Sha1::digest(&bytes).to_vec();
        if digest_trusted != self.hash {
            return false;
        }
        verify_block_difficulty(&self.block_inner, &self.hash[..])
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<BlockCompletePoW, BlockError> {
        let block: BlockCompletePoW = serde_json::from_slice(buffer).map_err(|_| BlockError::DeserializeError)?;
        if !block.is_valid() {
            Err(BlockError::InvalidBlock)
        } else {
            Ok(block)
        }
    }

    pub fn get_prev_hash(&self) -> &Vec<u8> {
        &self.block_inner.prev_hash
    }

    pub fn get_index(&self) -> &u32 {
        &self.block_inner.index
    }

    pub fn get_difficulty(&self) -> &u64 {
        &self.block_inner.difficulty
    }
}

impl BlockCompletePoS {

    pub fn is_valid(&self) -> bool {
        todo!("is Valid Block POS verify signature if signature is valid for signer id")
    }

    pub fn block_hash(&self) -> &[u8] {
        &self.signature[..]
    }

    pub fn get_prev_hash(&self) -> &Vec<u8> {
        &self.block_inner.prev_hash
    }

    pub fn get_index(&self) -> &u32 {
        &self.block_inner.index
    }

    pub fn get_difficulty(&self) -> &u64 {
        &self.block_inner.difficulty
    }
}


fn verify_block_difficulty(block: &Block, hash: &[u8]) -> bool {
    let hex = hex::encode(hash).into_bytes();
    let zeros = get_leading_zeros(&hex);
    zeros as u64 >= block.difficulty
}

fn compare_blockchains(ch1: &Chain, ch2: &Chain) -> () {
    let cmp = compare::natural();
    match cmp.compare(&ch1.get_number_of_blocks(), &ch2.get_number_of_blocks()) {
        Less | Greater => {
            Equal;
        }
        Equal => {
            Equal;
        }
    }
    todo!()
}

fn get_leading_zeros(array: &Vec<u8>) -> u8 {
    let mut result = 0u8;
    for u in array {
        if *u == 0 { result += 8; } else {
            let temp: u8 = u.leading_zeros().try_into().expect("Called leading zeros on a u8, got a value above 255");
            result += temp;
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_verify_block_difficulty() {
        eprintln!("O Sr. Andre esqueceu-se de implementar o teste");
        assert!(true)
    }
}