use std::fmt;
use std::fmt::{Formatter, LowerHex};
use seq_macro::seq;
use serde_derive::{Deserialize, Serialize};
use std::ops::{BitXor, Index, IndexMut};

#[derive(Serialize,Deserialize,Eq, PartialEq, Hash, Debug, Clone)]
pub struct KadID([u8; 20]);

impl KadID {
    pub(crate) fn zeroes() -> Self {
        KadID([0u8;20])
    }

    pub(crate) fn random() -> Self {
        use rand::{RngCore,rngs};
        let mut rng = rngs::ThreadRng::default();
        let mut val = [0u8; 20];
        rng.fill_bytes(&mut val);
        KadID(val)
    }

    // same as above, but falls within [2^i, 2^(i+1)[ distance of the origin,
    // therefore belonging to bucket i
    // This means, I want a random ID whose first i = 160 - i bits equal the origin, such that
    // out[..i] ^ origin[..i] == 0
    pub(crate) fn random_in_range(origin: &KadID, i: usize) -> Self {
        let i = 160 - i;
        //println!("===========");
        let mut res = Self::random();
        //println!("{:x}\ni={}", &res, i);
        let mut bit_count = 0u8;
        for (r_byte, o_byte) in res.0.iter_mut().zip(origin.as_ref().iter()) {
            //println!("{:x} / {:x}", &r_byte, &o_byte);
            if bit_count + 8 <= i as u8 {
                *r_byte = *o_byte;
                bit_count += 8;
            } else {
                let i = i as u8 - bit_count;
                let mut res = 0u8;
                let mut mask = 0b1111_1111;
                let mut select = 0b1000_0000;
                mask >>= i;
                select >>= i;
                res |= *r_byte & mask;
                res |= o_byte & !mask;

                res = if (o_byte & select) == 0 {
                    res | (1<<(7-i))
                } else {
                    res & !(1<<(7-i))
                };

                *r_byte = res;
                break;
            }
        }

        res
    }
}

impl TryFrom<&str> for KadID {
    type Error = hex::FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut res = [0u8;20];
        hex::decode_to_slice(value, &mut res)?;
        Ok(KadID(res))
    }
}

impl fmt::Display for KadID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s =  hex::encode_upper(self.0);
        f.write_str(&*s)
    }
}

impl LowerHex for KadID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for v in self.0 {
            f.write_fmt(format_args!("{:0<2x}", v))?
        }
        Ok(())
    }
}

impl BitXor for &KadID {
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

impl AsRef<[u8; 20]> for KadID {
    fn as_ref(&self) -> &[u8; 20] {
        &self.0
    }
}

pub fn bucket(distance: &[u8; 20]) -> u8 {
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


#[cfg(test)]
mod tests {
    use crate::kadid::{bucket, KadID};

    #[test]
    fn bucket_fn_tests() {
        let a = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(a), 0);
        let b = b"\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(b), 160);
        let c = b"\x00\x80\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(c), 152);
        let d = b"\x00\x80\x00\x00\x00\x00\x00\xFF\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(d), 152);
        let e = b"\x00\x70\x00\x00\x00\x00\x00\xFF\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert_eq!(bucket(e), 151);
    }

    #[test]
    fn random_in_range_dumb_test() {
        for _i in 0..10 {
            //println!("ITER {} ====================================", _i);
            let a = KadID::random();
            //println!("{:x}", &a);
            let b = KadID::random();
            //println!("{:x}", &b);
            //println!("d -> {:x}", KadID(&a^&b));
            let c = KadID::random_in_range(&a, bucket(&(&a ^ &b)) as usize);
            //println!("{:x}", &c);
            //println!("d -> {:x}", KadID(&a^&c));
            assert_eq!(bucket(&(&a ^ &c)), bucket(&(&a ^ &b)));
        }
    }

    #[test]
    fn random_in_range_less_dumb() {
        let a = KadID::random();
        for i in 0..=160 {
            let b = KadID::random_in_range(&a, i);
            assert_eq!(bucket(&(&a^&b)), i as u8)
        }
    }
}
