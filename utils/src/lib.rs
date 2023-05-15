use sha1::Digest;
use serde::{Deserialize, Serialize};

pub type HASH = Vec<u8>;



// [u8; 20]
pub fn get_hash<T :Serialize>(arr: &[T]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for i in arr.iter() {
        bytes.append(&mut bincode::serialize(i).unwrap());
    }
    sha1::Sha1::digest(&bytes).to_vec()
}