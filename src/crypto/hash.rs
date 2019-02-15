use std::fmt;

pub struct Hash256 {
    data: [u8; 32]
}

pub struct Hash8 {
    data: [u8; 8]
}

impl Hash256 {
    pub fn null_hash() -> Hash256 {
        Hash256 {
            data: [0;32]
        }
    }
    pub fn from(str: &str) -> Result<Hash256, hex::FromHexError> {
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
        let data = hex::decode(str)?;
        let mut hash = Hash8::null_hash();
        hash.data = array_ref!(data, 0, 8).clone();
        Ok(hash)
    }
}

macro_rules! impl_Display {
    (for $($t:ty),+) => {
        $(impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "<{}>", hex::encode(self.data))
            }
        })*
    }
}

impl_Display!(for Hash256, Hash8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_hash() {
        let hash = Hash256::null_hash();
        assert_eq!(hash.data, [0; 32]);
    }
}
