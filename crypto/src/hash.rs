use std::fmt;

#[derive(PartialEq)]
pub struct Hash256 {
    pub data: [u8; 32]
}

#[derive(PartialEq)]
pub struct Hash8 {
    pub data: [u8; 8]
}

impl Hash256 {
    pub fn null_hash() -> Hash256 {
        Hash256 {
            data: [0;32]
        }
    }
    pub fn from(str: &str) -> Result<Hash256, hex::FromHexError> {
        if str.len() != 64 {
            // TODO
            return Err(hex::FromHexError::OddLength);
        }
        let data = hex::decode(str)?;
        let mut hash = Hash256::null_hash();
        hash.data = array_ref!(data, 0, 32).clone();
        Ok(hash)
    }
}

impl Hash8 {
    pub fn null_hash() -> Hash8 {
        Hash8 {
            data: [0; 8]
        }
    }
    pub fn from(str: &str) -> Result<Hash8, hex::FromHexError> {
        if str.len() != 16 {
            return Err(hex::FromHexError::OddLength);
        }
        let data = hex::decode(str)?;
        let mut hash = Hash8::null_hash();
        hash.data = array_ref!(data, 0, 8).clone();
        Ok(hash)
    }
}

macro_rules! impl_Debug {
    (for $($t:ty),+) => {
        $(impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "<{}>", hex::encode(self.data))
            }
        })*
    }
}

impl_Debug!(for Hash256, Hash8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_hash() {
        let hash = Hash256::null_hash();
        assert_eq!(hash.data, [0; 32]);

        let hash = Hash8::null_hash();
        assert_eq!(hash.data, [0; 8]);
    }

    #[test]
    fn decodes_correctly() {
        let data: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let hash = Hash256::from("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
        assert_eq!(hash.data, data);

        let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let hash = Hash8::from("0102030405060708").unwrap();
        assert_eq!(hash.data, data);
    }

    #[test]
    fn errors_on_invalid_input() {
        assert!(Hash256::from("01").is_err());
        assert!(Hash8::from("01111111111111111111111111111111111111").is_err());
    }
}
