use sha1::Digest;
use std::fmt::{Debug};
use serde::{Deserialize, Serialize};
use auction_common::Transaction;
use utils::get_hash;

type MyHash = Vec<u8>; //[u8; 20];

// [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
//                              0
//              1                               2
//      3               4               5               6
//  7       8       9       10      11      12      13      14
//
#[derive(Serialize, Deserialize, Debug)]
pub struct MerkleTree {
    tree: Vec<MyHash>,
    depth: u32
}


impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            tree: Vec::new(),
            depth: 0
        }
    }

    pub fn from_transactions(mut transactions: Vec<Transaction>) -> Self {
        let depth = (transactions.len() as f32).log2().ceil() as u32;
        let max_leafs = (2_u32.pow(depth)) as usize;
        let n_internal_nodes = max_leafs - 1;
        // println!("Levels: {} | Internal: {} | Leafs: {} | Max Leafs: {}", levels, n_internal_nodes, transactions.len(), max_leafs);
        let mut tree: Vec<MyHash> = Vec::with_capacity(n_internal_nodes + max_leafs);
        transactions.sort();

        // Create all Nodes
        for _ in 0..(n_internal_nodes+max_leafs) {
            tree.push(vec![0; 20])
        }

        // Set values to leafs
        for (i, j) in transactions.iter().enumerate() {
            tree[n_internal_nodes + i] = get_hash(&[j])
        }

        // Set values to branches
        for i in (0..(n_internal_nodes)).rev() {
            tree[i] = get_hash(&[&tree[id_left(i)], &tree[id_right(i)]])
        }

        MerkleTree {
            tree,
            depth
        }
    }

    pub fn valid(&self) -> bool {
        valid_rec(self, 0, self.get_number_nodes())
    }

    pub fn get_tree(&self) -> &Vec<MyHash> {
        &self.tree
    }

    pub fn get_depth(&self) -> &u32 {
        &self.depth
    }

    pub fn get_number_nodes(&self) -> usize {
        (2_u32.pow(self.depth + 1) - 1) as usize
    }

    pub fn get_root_hash(&self) -> Option<MyHash> {
        if self.tree.is_empty() {
            return None
        }
        Some(self.tree[0].clone())
    }
}

fn id_left(parent: usize) -> usize {
    2*parent + 1
}

fn id_right(parent: usize) -> usize {
    2*parent + 2
}

pub fn valid_rec(tree: &MerkleTree, curr: usize, max: usize) -> bool {
    // is a leaf
    if curr >= (max / 2) {
        return true
    }
    let sons = [
        &tree.get_tree()[id_left(curr)],
        &tree.get_tree()[id_right(curr)]
    ];

    tree.get_tree()[curr] == get_hash(&sons) &&
        valid_rec(tree, id_left(curr), max) &&
        valid_rec(tree, id_right(curr), max)
}