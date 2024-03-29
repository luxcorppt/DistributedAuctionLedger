use std::fmt::{Debug};
use std::ops::Index;
use auction_common::{Transaction, TransactionSigned};
use utils::get_hash;

// type MyHash = [u8; 20]; //[u8; 20];

// [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
//                              0
//              1                               2
//      3               4               5               6
//  7       8       9       10      11      12      13      14
//
#[derive(Debug)]
pub struct MerkleTree<'a> {
    tree: Vec<[u8; 20]>,
    transactions: &'a Vec<TransactionSigned>,
    depth: u32
}


impl<'a> MerkleTree<'a> {
    pub fn from_transactions(transactions: &'a mut Vec<TransactionSigned>) -> Self {
        let depth = (transactions.len() as f32).log2().ceil() as u32;
        let max_leafs = (2_u32.pow(depth)) as usize;
        let n_internal_nodes = max_leafs - 1;
        // println!("Levels: {} | Internal: {} | Leafs: {} | Max Leafs: {}", depth, n_internal_nodes, transactions.len(), max_leafs);
        let mut tree: Vec<[u8; 20]> = vec![[0; 20]; n_internal_nodes + max_leafs];
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
        valid_rec(self, 0, self.get_number_nodes(),
                  |i, hash| get_hash(&[&self.transactions[i]]) == *hash)
    }

    pub fn get_depth(&self) -> &u32 {
        &self.depth
    }

    pub fn get_number_nodes(&self) -> usize {
        (2_u32.pow(self.depth + 1) - 1) as usize
    }

    pub fn get_root_hash(&self) -> Option<[u8; 20]> {
        if self.tree.is_empty() {
            return None
        }
        Some(self.tree[0].clone())
    }
}

impl<'a> Index<usize> for MerkleTree<'a> {
    type Output = [u8; 20];

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
    F: Fn(usize, &[u8; 20]) -> bool + Clone
{
    // is a leaf Verify if transaction m
    if curr >= (max / 2) {
        return curr - (max / 2) >= tree.transactions.len() || leaf_verifier(curr - (max / 2), &tree[curr])
    }
    let sons = [
        &tree[id_left(curr)],
        &tree[id_right(curr)]
    ];

    tree[curr] == get_hash(&sons) &&
        valid_rec(tree, id_left(curr), max, leaf_verifier.clone()) &&
        valid_rec(tree, id_right(curr), max, leaf_verifier)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek_fiat::{Keypair, Signer};
    use rand::rngs;
    use auction_common::{Transaction, TransactionSemiSigned, TransactionSigned};
    use crate::{merkle};

    fn sign_sender(transaction: Transaction, keypair: &Keypair) -> TransactionSemiSigned {
        let sig = keypair.sign(&transaction.body_serialize().expect("Error signing")[..]);
        transaction.sign(sig)
    }

    fn sign_receiver(transaction: TransactionSemiSigned, keypair: &Keypair) -> TransactionSigned {
        let sig = keypair.sign(&transaction.body_serialize().expect("Error signing2")[..]);
        transaction.sign(sig).expect("Error completing signing")
    }

    #[test]
    fn test_merkle_tree() {
        let mut rng = rngs::ThreadRng::default();
        let sender = Keypair::generate(&mut rng);
        let receiver = Keypair::generate(&mut rng);
        let mut transactions = Vec::new();
        for i in 0..45 {
            transactions.push(Transaction::new(sender.public, receiver.public, i));
        }
        let transactions: Vec<_> = transactions.into_iter().map(|x| sign_sender(x, &sender)).collect();
        let mut transactions: Vec<_> = transactions.into_iter().map(|x| sign_receiver(x, &receiver)).collect();

        let tree = merkle::MerkleTree::from_transactions(&mut transactions);

        println!("{:?}", tree);
        assert!(tree.valid())
    }

    #[test]
    fn test_merkle_tree2() {
        let mut rng = rngs::ThreadRng::default();
        let sender = Keypair::generate(&mut rng);
        let receiver = Keypair::generate(&mut rng);
        let mut transactions = Vec::new();
        for i in 0..65 {
            transactions.push(Transaction::new(sender.public, receiver.public, i));
        }
        let mut transactions: Vec<_> = transactions.into_iter().map(|x| sign_sender(x, &sender)).collect();
        let mut transactions: Vec<_> = transactions.into_iter().map(|x| sign_receiver(x, &receiver)).collect();

        let tree = merkle::MerkleTree::from_transactions(&mut transactions);

        println!("{:?}", tree);
        assert!(tree.valid())
    }

    #[test]
    fn test_merkle_tree3() {
        let mut rng = rngs::ThreadRng::default();
        let sender = Keypair::generate(&mut rng);
        let receiver = Keypair::generate(&mut rng);
        let mut transactions = Vec::new();
        for i in 0..7 {
            transactions.push(Transaction::new(sender.public, receiver.public, i));
        }
        let transactions: Vec<_> = transactions.into_iter().map(|x| sign_sender(x, &sender)).collect();
        let mut transactions: Vec<_> = transactions.into_iter().map(|x| sign_receiver(x, &receiver)).collect();

        let tree = merkle::MerkleTree::from_transactions(&mut transactions);

        println!("{:?}", tree);
        assert!(tree.valid())
    }
}