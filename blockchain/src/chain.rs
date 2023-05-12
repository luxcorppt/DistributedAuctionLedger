use crate::block::BlockComplete;
use crate::chain::ChainError::NotNextBlock;

pub enum ChainError {
    NotNextBlock
}

pub struct Chain {
    chain: Vec<BlockComplete>
}

impl Chain {
    pub fn add(&mut self, block: BlockComplete) -> Result<(), ChainError> {
        if let Some(prev) = self.chain.last() {
            let prev = prev.block_hash();
            if block.as_block().prev_hash != prev {
                return Err(NotNextBlock)
            }
        }
        self.chain.push(block);
        Ok(())
    }
}