use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use auction_common::Transaction;

pub enum BlockError {
    DeserializeError,
    InvalidBlock
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockComplete {
    hash: Vec<u8>,
    block_inner: Block
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block{
    pub index: u32,
    pub timestamp: u128,
    pub prev_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub difficulty: u64
}

impl Block {
    pub fn new(index: u32, timestamp: u128, prev_hash: Vec<u8>, transactions: Vec<Transaction>, difficulty: u64) -> Self {
        Block {
            index, timestamp, prev_hash, transactions, nonce: 0, difficulty
        }
    }

    pub fn complete(mut self) -> BlockComplete {
        loop {
            let bytes = bincode::serialize(&self).unwrap();
            let digest = sha1::Sha1::digest(&bytes);
            if verify_block_difficulty(&self, &digest[..]) {
                return BlockComplete {
                    hash: digest.to_vec(),
                    block_inner: self
                }
            }
            self.nonce += 1
        }
    }
}

impl BlockComplete {
    pub fn as_block(&self) -> &Block {
        &self.block_inner
    }

    pub fn block_hash(&self) -> &[u8] {
        &self.hash[..]
    }

    fn validate(&self) -> bool {
        let bytes = bincode::serialize(&self.block_inner).unwrap();
        let digest_trusted = sha1::Sha1::digest(&bytes).to_vec();
        if digest_trusted != self.hash {
            return false;
        }
        verify_block_difficulty(&self.block_inner, &self.hash[..])
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<BlockComplete, BlockError> {
        let block: BlockComplete = serde_json::from_slice(buffer).map_err(|_| BlockError::DeserializeError)?;
        if !block.validate() {
            Err(BlockError::InvalidBlock)
        } else {
            Ok(block)
        }
    }
}

fn verify_block_difficulty(block: &Block, hash: &[u8]) -> bool {
    let hex = hex::encode(hash).into_bytes();
    hex[..block.difficulty as usize].into_iter().all(|&b| b == 0x00)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_verify_block_difficulty() {
        eprintln!("O Sr. Andre esqueceu-se de implementar o teste");
        assert!(true)
    }
}