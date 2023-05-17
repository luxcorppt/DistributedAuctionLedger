use std::cmp::{min, Ordering};
use crate::block::{Block, BlockComplete};
use crate::chain::ChainError::{NotNextBlock, NotValidBlock};
use std::cmp::Ordering::{Equal, Greater, Less};

use serde::{Deserialize, Serialize};
use std::fmt::{Debug};
use std::slice::Iter;


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
            if block.get_prev_hash() != prev {
                return Err(NotNextBlock)
            }
        }
        if !block.is_valid() {
            return Err(NotValidBlock)
        }
        self.chain.push(block);
        Ok(())
    }

    pub fn iter(&self) -> Iter<'_, BlockComplete> {
        self.chain.iter()
    }

    pub fn find(&self, hash: &[u8]) -> Option<&BlockComplete> {
        self.chain.iter().find(|b| {
            b.block_hash() == hash
        })
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

    pub fn get(&self, index: usize) -> Option<&BlockComplete> {
        if index >= self.chain.len() {
            return None
        }
        Some(&self.chain[index])
    }

    pub fn get_number_of_blocks(&self) -> usize {
        self.chain.len()
    }
}

fn compare_chains(ch1: &Chain, ch2: &Chain) -> Ordering {
     match ch1.get_number_of_blocks().cmp(&ch2.get_number_of_blocks()) {
        Less => { Greater }
        Greater => { Less }
        Equal => {
            Equal
        }
    }
}
