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

pub fn get_leading_zeros(array: &Vec<u8>) -> u8 {
    let mut result = 0u8;
    for u in array {
        if *u == 0 { result += 8; } else {
            let temp: u8 = u.leading_zeros().try_into().expect("Called leading zeros on a u8, got a value above 255");
            result += temp;
            break;
        }
    }
    result
}
