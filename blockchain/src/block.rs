use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use auction_common::Transaction;
use utils::get_hash;
use crate::merkle::MerkleTree;

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
    merkle: Vec<u8>,
    difficulty: u64
}

impl Block {
    pub fn new(index: u32, timestamp: u128, prev_hash: Vec<u8>, transactions: Vec<Transaction>, difficulty: u64) -> Self {
        match MerkleTree::from_transactions(transactions.clone()).get_root_hash() {
            None => {
                Block {
                    index, timestamp, prev_hash, nonce: 0, transactions, merkle: Vec::new(), difficulty
                }
            }
            Some(merkle) => {
                Block {
                    index, timestamp, prev_hash, nonce: 0, transactions, merkle, difficulty
                }
            }
        }

    }

    pub fn complete_pow(mut self) -> BlockCompletePoW {
        loop {
            let digest = get_hash(&[&self]);
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
        if get_hash(&[&self.block_inner]) != self.hash {
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
    let zeros = utils::get_leading_zeros(&hex);
    zeros as u64 >= block.difficulty
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_verify_block_difficulty() {
        eprintln!("O Sr. Andre esqueceu-se de implementar o teste");
        assert!(true)
    }
}