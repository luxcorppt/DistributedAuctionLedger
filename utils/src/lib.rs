use rand::{RngCore, rngs};
use serde::{Deserialize, Serialize};

// [u8; 20]
pub fn get_hash<T :Serialize>(arr: &[T]) -> [u8; 20] {
    let mut hasher = sha1_smol::Sha1::new();
    for i in arr.iter() {
        hasher.update(&mut bincode::serialize(i).unwrap());
    }
    hasher.digest().bytes()
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

pub fn random() -> Vec<u8> {
    let mut rng = rngs::ThreadRng::default();
    let mut val = [0u8; 20];
    rng.fill_bytes(&mut val);
    val.to_vec()
}