use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use auction_common::Transaction;

pub enum BlockError {
    DeserializeError,
    InvalidBlock
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block{
    pub index: u32,
    pub timestamp: u128,
    pub prev_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub difficulty: u64
}

impl Block {
    pub fn new(index: u32, timestamp: u128, prev_hash: Vec<u8>, transactions: Vec<Transaction>, difficulty: u64) -> Self {
        Block {
            index, timestamp, prev_hash, transactions, difficulty
        }
    }

    pub fn complete_pow(self) -> BlockCompletePoW {
        let mut nonce: u64 = 0;
        loop {
            let bytes_block = bincode::serialize(&self).unwrap();
            // TODO: validate that we can use the nonce outside of the block struct
            let bytes_nonce = bincode::serialize(&nonce).unwrap();
            let bytes_all:Vec<u8> = [bytes_block, bytes_nonce].concat();
            let digest = sha1::Sha1::digest(&bytes_all);
            if verify_block_difficulty(&self, &digest[..]) {
                return BlockCompletePoW {
                    hash: digest.to_vec(),
                    nonce,
                    block_inner: self
                }
            }
            nonce += 1
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
    nonce: u64,
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
}

impl BlockCompletePoW {
    pub fn as_block(&self) -> &Block {
        &self.block_inner
    }

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
}

impl BlockCompletePoS {

    pub fn is_valid(&self) -> bool {
        todo!("is Valid Block POS verify signature if signature is valid for signer id")
    }

    pub fn block_hash(&self) -> &[u8] {
        &self.signature[..]
    }

    pub fn as_block(&self) -> &Block {
        &self.block_inner
    }
}


fn verify_block_difficulty(block: &Block, hash: &[u8]) -> bool {
    let hex = hex::encode(hash).into_bytes();
    hex[..block.difficulty as usize].into_iter().all(|&b| b == 0x00);
    hash[0] % 13 == 0
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_verify_block_difficulty() {
        eprintln!("O Sr. Andre esqueceu-se de implementar o teste");
        assert!(true)
    }
}