use seq_macro::seq;
use std::ops::{Index, IndexMut};

pub fn xor_bytes(lhs: &[u8;20], rhs: &[u8;20]) -> [u8; 20] {
    let mut result = [0u8; 20];
    seq!(N in 0..20 {
        *result.index_mut(N) = lhs.index(N) ^ rhs.index(N);
    });
    result
}

pub fn bucket(distance: &[u8; 20]) -> u8 {
    let result = get_leading_zeros(distance);
    160 - result
}

pub fn get_leading_zeros(array: &[u8; 20]) -> u8 {
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
