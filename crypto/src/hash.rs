use std::convert::{From, Into, TryFrom};
use std::fmt::{Display, Formatter};

use digest::Digest;
use serde::{Deserialize, Serialize};
use sha3::Keccak256Full;

/// Type alias for compatibility with the digest crate
pub type Hash256Data = generic_array::GenericArray<u8, generic_array::typenum::U32>;

/// Type alias for compatibility with the digest crate
pub type Hash8Data = generic_array::GenericArray<u8, generic_array::typenum::U8>;

/// Representation of 256-bit hash values
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Hash256(Hash256Data);

/// Representation of 64-bit hash values
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Hash8(Hash8Data);

impl Hash256 {
    /// Gets a null hash (32 0-bytes)
    pub fn null_hash() -> Self {
        Hash256::from(Hash256Data::from([0; 32]))
    }
    /// Gets a reference to the internal byte buffer
    pub fn data(&self) -> &Hash256Data {
        &self.0
    }
}

impl Display for Hash256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<Hash256Data> for Hash256 {
    fn from(data: Hash256Data) -> Self {
        Hash256(data)
    }
}

impl Into<String> for Hash256 {
    fn into(self) -> String {
        hex::encode(self.0)
    }
}

impl TryFrom<&str> for Hash256 {
    type Error = hex::FromHexError;
    fn try_from(data: &str) -> Result<Self, Self::Error> {
        if data.len() != 64 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        Ok(Hash256(Hash256Data::clone_from_slice(&hex::decode(data)?)))
    }
}

impl Hash8 {
    /// Gets a null hash (8 0-bytes)
    pub fn null_hash() -> Self {
        Hash8::from(Hash8Data::from([0; 8]))
    }
    /// Gets a reference to the internal byte buffer
    pub fn data(&self) -> &Hash8Data {
        &self.0
    }
}

impl Display for Hash8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<Hash8Data> for Hash8 {
    fn from(data: Hash8Data) -> Self {
        Hash8(data)
    }
}

impl Into<String> for Hash8 {
    fn into(self) -> String {
        hex::encode(self.0)
    }
}

impl TryFrom<&str> for Hash8 {
    type Error = hex::FromHexError;
    fn try_from(data: &str) -> Result<Self, Self::Error> {
        if data.len() != 16 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        Ok(Hash8(Hash8Data::clone_from_slice(&hex::decode(data)?)))
    }
}

/// Alias of Keccak finalist to keep similarity with the C++ version
pub struct CNFastHash {
    hasher: Keccak256Full,
}

impl Digest for CNFastHash {
    type OutputSize = digest::generic_array::typenum::U32;
    fn new() -> Self {
        CNFastHash {
            hasher: Keccak256Full::new(),
        }
    }
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.hasher.input(data);
    }
    fn chain<B: AsRef<[u8]>>(self, data: B) -> Self {
        CNFastHash {
            hasher: self.hasher.chain(data),
        }
    }
    fn result(self) -> Hash256Data {
        *Hash256Data::from_slice(&self.hasher.result()[..32])
    }
    fn result_reset(&mut self) -> Hash256Data {
        *Hash256Data::from_slice(&self.hasher.result_reset()[..32])
    }
    fn reset(&mut self) {
        self.hasher.reset()
    }
    fn digest(data: &[u8]) -> Hash256Data {
        *Hash256Data::from_slice(&Keccak256Full::digest(data)[..32])
    }
    fn output_size() -> usize {
        32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_hash() {
        assert_eq!(
            Hash256::null_hash().to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn decodes_correctly() {
        let data: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let hash =
            Hash256::try_from("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20")
                .unwrap();
        assert_eq!(hash.data().as_slice(), data);
    }

    #[test]
    fn errors_on_invalid_input() {
        assert!(Hash256::try_from("01").is_err());
    }
}
