use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use auction_common::Transaction;

pub enum BlockError {
    DeserializeError,
    InvalidBlock
}

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

    pub fn complete_pow(mut self) -> BlockCompletePoW {
        loop {
            let bytes = bincode::serialize(&self).unwrap();
            let digest = sha1::Sha1::digest(&bytes);
            if verify_block_difficulty(&self, &digest[..]) {
                return BlockCompletePoW {
                    hash: digest.to_vec(),
                    block_inner: self
                }
            }
            self.nonce += 1
        }
    }

    pub fn complete_pos(mut self) -> BlockCompletePoS {
        todo!()
    }
}

impl BlockCompletePoW {
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

    pub fn from_bytes(buffer: &[u8]) -> Result<BlockCompletePoW, BlockError> {
        let block: BlockCompletePoW = serde_json::from_slice(buffer).map_err(|_| BlockError::DeserializeError)?;
        if !block.validate() {
            Err(BlockError::InvalidBlock)
        } else {
            Ok(block)
        }
    }
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
    pub fn as_block(&self) -> &Block {
        match self {
            BlockComplete::POW(block) => {
                block.as_block()
            }
            BlockComplete::POS(block) => {
                block.as_block()
            }
        }
    }
}

impl BlockCompletePoS {

    pub fn is_valid() -> bool {
        todo!()
    }

    pub fn block_hash(&self) -> &[u8] {
        todo!()
    }

    pub fn as_block(&self) -> &Block {
        todo!()
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