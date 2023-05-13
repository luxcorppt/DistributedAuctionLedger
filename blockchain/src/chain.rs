use crate::block::{BlockComplete};
use crate::chain::ChainError::{NotNextBlock, NotValidBlock};

use serde::{Deserialize, Serialize};
use std::fmt::{Debug};


#[derive(Serialize, Deserialize, Debug)]
pub enum ChainError {
    NotNextBlock,
    NotValidBlock
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chain {
    chain: Vec<BlockComplete>
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            chain: Vec::new(),
        }
    }
    pub fn add(&mut self, block: BlockComplete) -> Result<(), ChainError> {
        if let Some(prev) = self.chain.last() {
            let prev = prev.block_hash();
            if block.as_block().prev_hash != prev {
                return Err(NotNextBlock)
            }
        }
        if !block.is_valid() {
            return Err(NotValidBlock)
        }
        self.chain.push(block);
        Ok(())
    }

    pub fn get_last(&self) -> Option<&BlockComplete> {
        self.chain.last()
    }

    pub fn debug(&self) {
        println!("\n\n");
        for (n, block) in self.chain.iter().enumerate() {
            println!("Block {}: {:?}", n, block)
        }
        println!("\n\n");
    }
}