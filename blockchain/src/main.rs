

mod block;
mod chain;
mod clients;
mod merkle;

fn main() {

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
    fn test_merkle_tree() {
        let mut transactions = Vec::from([
            Transaction::new(0),
            Transaction::new(1),
            Transaction::new(2),
            Transaction::new(3),
            Transaction::new(4),
            Transaction::new(5),
            Transaction::new(6),
            Transaction::new(7),
            Transaction::new(8),
            Transaction::new(9),
            Transaction::new(10),
            Transaction::new(11),
            Transaction::new(12),
            Transaction::new(13),
            Transaction::new(14),
            Transaction::new(15)
        ]);
        let tree = merkle::MerkleTree::from_transactions(&mut transactions);

        println!("{:?}", tree);
        assert!(tree.valid())
    }

    #[test]
    fn test_blockchain_pos() {
        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let block = Block::new(1, tm, [5; 20], vec![], 1);


        let mut rng = rngs::ThreadRng::default();
        let keypair = Keypair::generate(&mut rng);
        let hash = get_hash(&[&block]);
        let bytes = bincode::serialize(&hash).unwrap();

        let sig = keypair.sign(&bytes[..]);

        let block = block.complete_pos(keypair.public, sig);
        println!("{:?}", block);
        let mut chain = chain::Chain::new();
        chain.add(BlockComplete::POS(block)).expect("Error Adding first block");
        println!("{:?}\n\n\n", chain);
        chain.debug();
        assert_eq!(chain.get_number_of_blocks(), 1);
    }

    #[test]
    fn test_blockchain_pow() {
        let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        let block = Block::new(1, tm, [255; 20], vec![], 2);
        let block = block.complete_pow();
        println!("{:?}", block);
        let mut chain = chain::Chain::new();
        chain.add(BlockComplete::POW(block)).expect("Error Adding first block");
        // println!("{:?}\n\n\n", chain);
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