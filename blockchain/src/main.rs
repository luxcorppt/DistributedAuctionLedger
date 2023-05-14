use std::time::SystemTime;
use crate::block::{Block, BlockComplete};

mod block;
mod chain;
mod clients;
mod merkle;

fn main() {
    // let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    // let block = Block::new(1, tm, vec![5; 8], vec![], 1);
    //
    //
    // let keypair = rcgen::KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    // let pk = keypair.public_key_raw();
    // let bytes = bincode::serialize(&block).unwrap();
    //
    // Sign
    //
    // let block = block.complete_pos(vec![], vec![]);
    // println!("{:?}", block);
    // let mut chain = chain::Chain::new();
    // chain.add(BlockComplete::POS(block)).expect("Error Adding first block");
    // println!("{:?}\n\n\n", chain);
    // chain.debug();
    //
    // return;

    let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let block = Block::new(1, tm, vec![5; 8], vec![], 1);
    let block = block.complete_pow();
    println!("{:?}", block);
    let mut chain = chain::Chain::new();
    chain.add(BlockComplete::POW(block)).expect("Error Adding first block");
    // println!("{:?}\n\n\n", chain);
    chain.debug();

    let blk = chain.get_last().unwrap();
    let tm = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let new_block = Block::new(blk.get_index()+1, tm, Vec::from(blk.block_hash()), vec![], blk.get_difficulty().clone());
    let new_block = new_block.complete_pow();
    println!("{:?}", new_block);
    chain.add(BlockComplete::POW(new_block)).expect("Error Adding second block");
    // println!("{:?}\n\n\n", chain);
    chain.debug();


}
