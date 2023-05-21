use std::cmp::{Ordering};
use crate::block::{BlockComplete};
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


#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use ed25519_dalek_fiat::{Keypair, Signer};
    use rand::rngs;
    use auction_common::Transaction;
    use utils::get_hash;
    use crate::block::{Block, BlockComplete};
    use crate::{chain, merkle};

    #[test]
    fn test_blockchain_pos() {
        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let block = Block::new(1, tm, [5; 20], vec![], 1);

        let mut rng = rngs::ThreadRng::default();
        let keypair = Keypair::generate(&mut rng);
        let hash = get_hash(&[&block]);
        let bytes = bincode::serialize(&hash).unwrap();

        let sig = keypair.sign(&bytes[..]);

        let block_complete1 = block.complete_pos(keypair.public, sig, 50);
        println!("Block 1: {:?}", block_complete1);

        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let block = Block::new(block_complete1.get_index() + 1, tm, *block_complete1.block_hash(), vec![], 1);

        let mut rng = rngs::ThreadRng::default();
        let keypair = Keypair::generate(&mut rng);
        let hash = get_hash(&[&block]);
        let bytes = bincode::serialize(&hash).unwrap();

        let sig = keypair.sign(&bytes[..]);

        let block_complete2 = block.complete_pos(keypair.public, sig, 50);
        println!("Block 2: {:?}", block_complete2);

        let mut chain = chain::Chain::new();
        chain.add(BlockComplete::POS(block_complete1)).expect("Error Adding first block");
        println!("{:?}\n\n\n", chain);

        chain.add(BlockComplete::POS(block_complete2)).expect("Error Adding second block");
        println!("{:?}\n\n\n", chain);
        chain.debug();
        assert_eq!(chain.get_number_of_blocks(), 2);
    }

    #[test]
    fn test_blockchain_pow() {
        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let block = Block::new(1, tm, [255; 20], vec![], 1);
        let block = block.complete_pow();
        println!("{:?}", block);
        let mut chain = chain::Chain::new();
        chain.add(BlockComplete::POW(block)).expect("Error Adding first block");
        println!("{:?}\n\n\n", chain);
        chain.debug();

        let blk = chain.get_last().unwrap();
        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let new_block = Block::new(blk.get_index() + 1, tm, *blk.block_hash(), vec![], blk.get_difficulty().clone());
        let new_block = new_block.complete_pow();
        println!("{:?}", new_block);
        chain.add(BlockComplete::POW(new_block)).expect("Error Adding second block");
        // println!("{:?}\n\n\n", chain);
        chain.debug();
        assert_eq!(chain.get_number_of_blocks(), 2);
    }
}