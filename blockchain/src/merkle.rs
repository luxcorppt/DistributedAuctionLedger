use sha1::Digest;
use std::fmt::{Debug};
use std::ops::Index;
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
#[derive(Debug)]
pub struct MerkleTree<'a> {
    tree: Vec<MyHash>,
    transactions: &'a Vec<Transaction>,
    depth: u32
}


impl<'a> MerkleTree<'a> {
    pub fn from_transactions(transactions: &'a mut Vec<Transaction>) -> Self {
        let depth = (transactions.len() as f32).log2().ceil() as u32;
        let max_leafs = (2_u32.pow(depth)) as usize;
        let n_internal_nodes = max_leafs - 1;
        // println!("Levels: {} | Internal: {} | Leafs: {} | Max Leafs: {}", depth, n_internal_nodes, transactions.len(), max_leafs);
        let mut tree: Vec<MyHash> = vec![vec![0; 20]; n_internal_nodes + max_leafs];
        transactions.sort_unstable();

        // Set values to leafs
        for (i, j) in transactions.iter().enumerate() {
            tree[n_internal_nodes + i] = get_hash(&[j])
        }

        // Set values to branches
        for i in (0..n_internal_nodes).rev() {
            tree[i] = get_hash(&[&tree[id_left(i)], &tree[id_right(i)]])
        }

        MerkleTree {
            tree,
            depth,
            transactions
        }
    }

    pub fn valid(&self) -> bool {
        valid_rec(self, 0, self.get_number_nodes(), |i, hash| {
            get_hash(&[&self.transactions[i]]) == *hash
        })
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

impl<'a> Index<usize> for MerkleTree<'a> {
    type Output = MyHash;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tree[index]
    }
}

fn id_left(parent: usize) -> usize {
    2*parent + 1
}

fn id_right(parent: usize) -> usize {
    2*parent + 2
}

pub fn valid_rec<F>(tree: &MerkleTree, curr: usize, max: usize, leaf_verifier: F) -> bool
where
    F: Fn(usize, &MyHash) -> bool + Clone
{
    // is a leaf Verify if transaction m
    if curr >= (max / 2) {
        return leaf_verifier(curr - (max / 2), &tree[curr])
    }
    let sons = [
        &tree[id_left(curr)],
        &tree[id_right(curr)]
    ];

    tree[curr] == get_hash(&sons) &&
        valid_rec(tree, id_left(curr), max, leaf_verifier.clone()) &&
        valid_rec(tree, id_right(curr), max, leaf_verifier)
}