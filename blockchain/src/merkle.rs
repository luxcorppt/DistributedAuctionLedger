use sha1::Digest;
use enum_dispatch::enum_dispatch;

type MyHash = Vec<u8>;

pub struct MerkleNode {
    hash: MyHash,
    left: Box<Option<MerkleNode>>,
    right: Box<Option<MerkleNode>>
}

enum Node {
    Leaf(MerkleNode),
    Branch(MerkleNode)
}

impl MerkleNode {
    pub fn empty(hash: MyHash) -> Self {
        MerkleNode {
            hash,
            left: Box::new(None),
            right: Box::new(None)
        }
    }

    pub fn new(left: MerkleNode, right: MerkleNode) -> Self {
        let mut bytes : Vec<u8> = Vec::new();
        bytes.append(&mut bincode::serialize(left.get_hash()).unwrap());
        bytes.append(&mut bincode::serialize(right.get_hash()).unwrap());

        let digest = sha1::Sha1::digest(&bytes).to_vec();
        MerkleNode {
            hash: digest,
            left: Box::new(Some(left)),
            right: Box::new(Some(right)),

        }
    }

    pub fn get_hash(&self) -> &[u8] {
        &self.hash[..]
    }
}

pub struct MerkleTree {
    nodes: Vec<MyHash>
}