#![allow(dead_code)]

mod rpc;
mod node;

use std::ops::{BitXor, Index, IndexMut};
use seq_macro::seq;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

type Result<T> = std::result::Result<T, KadError>;

#[derive(Serialize,Deserialize,Eq, PartialEq, Hash)]
struct KadID([u8; 20]);

impl KadID {
    fn random() -> Self {
        use rand::{RngCore,rngs};
        let mut rng = rngs::ThreadRng::default();
        let mut val = [0u8; 20];
        rng.fill_bytes(&mut val);
        KadID(val)
    }
}

impl BitXor for KadID {
    type Output = [u8; 20];

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut result = [0u8; 20];
        seq!(N in 0..20 {
            *result.index_mut(N) = self.0.index(N) ^ rhs.0.index(N);
        });
        result
    }
}

/// The max bucket size for a given node
const K: u8 = 3;

fn bucket(distance: &[u8; 20]) -> u8 {
    let mut result = 0u8;
    for u in distance {
        if *u == 0 { result += 8; }
        else {
            let temp: u8 = u.leading_zeros().try_into().expect("Called leading zeros on a u8, got a value above 255");
            result += temp;
            break;
        }
    }
    160 - result
}

#[derive(Error, Debug)]
enum KadError {
    #[error("Failed to deserialize Node data")]
    LocalDeserializeError(serde_json::Error),
    #[error("Failed to serialize protocol message")]
    MessageSerializeError(serde_json::Error),
    #[error("Failed to deserialize protocol message")]
    MessageDeserializeError(serde_json::Error),
}



#[cfg(test)]
mod tests {
    use crate::bucket;

    #[test]
    fn bucket_fn_tests() {
        let a = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(a), 0);
        let b = b"\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(b), 160);
        let c = b"\x00\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(c), 152);
        let c = b"\x00\x80\x00\x00\x00\x00\x00\xFF\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(c), 152);
    }
}