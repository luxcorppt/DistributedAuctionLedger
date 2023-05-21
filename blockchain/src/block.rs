#![allow(unused)]

use std::fmt::Debug;
use ed25519_dalek_fiat::{Signature, PublicKey, Verifier, Keypair};
use serde::{Deserialize, Serialize};
use auction_common::Transaction;
use utils::{get_hash};
use crate::merkle::MerkleTree;
use rand::rngs;


pub enum BlockError {
    DeserializeError,
    InvalidBlock
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block{
    index: u32,
    timestamp: u128,
    prev_hash: [u8; 20],
    nonce: u64,
    transactions: Vec<Transaction>,
    merkle: [u8; 20],
    difficulty: u64
}

impl Block {
    pub fn new(index: u32, timestamp: u128, prev_hash: [u8; 20], mut transactions: Vec<Transaction>, difficulty: u64) -> Self {
        match MerkleTree::from_transactions(&mut transactions).get_root_hash() {
            None => {
                Block {
                    index, timestamp, prev_hash, nonce: 0, transactions, merkle: [0; 20], difficulty
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
            if verify_block_difficulty(&self, &digest) {
                return BlockCompletePoW {
                    hash: digest,
                    block_inner: self
                }
            }
            self.nonce += 1
        }
    }

    // pub fn complete_pos(self, signing_id: &[u8; 32], signature: &[u8; 64]) -> BlockCompletePoS {
    pub fn complete_pos(self, signing_id: PublicKey, signature: Signature) -> BlockCompletePoS {
        BlockCompletePoS {
            signing_id,
            signature,
            hash: get_hash(&[&self]),
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
    hash: [u8; 20],
    block_inner: Block
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockCompletePoS {
    signature: Signature,
    signing_id: PublicKey,
    hash: [u8; 20],
    block_inner: Block
}

impl BlockComplete {
    pub fn block_hash(&self) -> &[u8; 20] {
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

    pub fn get_prev_hash(&self) -> &[u8; 20]  {
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
    pub fn block_hash(&self) -> &[u8; 20] {
        &self.hash
    }

    fn is_valid(&self) -> bool {
        if get_hash(&[&self.block_inner]) != self.hash {
            return false;
        }
        verify_block_difficulty(&self.block_inner, &self.hash)
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<BlockCompletePoW, BlockError> {
        let block: BlockCompletePoW = serde_json::from_slice(buffer).map_err(|_| BlockError::DeserializeError)?;
        if !block.is_valid() {
            Err(BlockError::InvalidBlock)
        } else {
            Ok(block)
        }
    }

    pub fn get_prev_hash(&self) -> &[u8; 20] {
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
        let mut rng = rngs::ThreadRng::default();
        let keypair = Keypair::generate(&mut rng);
        let bytes = &bincode::serialize(&self.hash).unwrap()[..];
        match self.signing_id.verify(bytes, &self.signature) {
            Ok(_) => {
                true
            }
            _ => {
                false
            }
        }
    }

    pub fn block_sig(&self) -> &Signature {
        &self.signature
    }

    pub fn block_hash(&self) -> &[u8; 20] {
        &self.hash
    }

    pub fn get_prev_hash(&self) -> &[u8; 20] {
        &self.block_inner.prev_hash
    }

    pub fn get_index(&self) -> &u32 {
        &self.block_inner.index
    }

    pub fn get_difficulty(&self) -> &u64 {
        &self.block_inner.difficulty
    }
}


fn verify_block_difficulty(block: &Block, hash: &[u8; 20]) -> bool {
    let zeros = utils::get_leading_zeros(hash);
    zeros as u64 >= block.difficulty
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;
    use utils::get_hash;
    use crate::block::{Block, verify_block_difficulty};

    #[test]
    fn verify_block_diff(){
        let block = Block::new(1, 0, [0; 20], vec![], 1);
        assert!(verify_block_difficulty(&block, &[0u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8]));
        assert!(!verify_block_difficulty(&block, &[255u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8]));
        assert!(verify_block_difficulty(&block, &[127u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,255u8]));
        assert!(!verify_block_difficulty(&block, &[128u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8,5u8]));
    }
}