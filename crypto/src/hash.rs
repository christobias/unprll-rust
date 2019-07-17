use digest::Digest;
use sha3::Keccak256Full;

pub type Hash256 = generic_array::GenericArray<u8, generic_array::typenum::U32>;
pub type Hash8 = generic_array::GenericArray<u8, generic_array::typenum::U1>;

pub struct CNFastHash {
    hasher: Keccak256Full
}

impl Digest for CNFastHash {
    type OutputSize = digest::generic_array::typenum::U32;
    fn new() -> Self {
        CNFastHash {
            hasher: Keccak256Full::new()
        }
    }
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.hasher.input(data);
    }
    fn chain<B: AsRef<[u8]>>(self, data: B) -> Self {
        CNFastHash {
            hasher: self.hasher.chain(data)
        }
    }
    fn result(self) -> Hash256 {
        *Hash256::from_slice(&self.hasher.result()[..32])
    }
    fn result_reset(&mut self) -> Hash256 {
        *Hash256::from_slice(&self.hasher.result_reset()[..32])
    }
    fn reset(&mut self) {
        self.hasher.reset()
    }
    fn digest(data: &[u8]) -> Hash256 {
        *Hash256::from_slice(&Keccak256Full::digest(data))
    }
    fn output_size() -> usize {
        return 32;
    }
}

//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn null_hash() {
//         let hash = Hash256::null_hash();
//         assert_eq!(hash.data, [0; 32]);
//
//         let hash = Hash8::null_hash();
//         assert_eq!(hash.data, [0; 8]);
//     }
//
//     #[test]
//     fn decodes_correctly() {
//         let data: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
//         let hash = Hash256::try_from("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
//         assert_eq!(hash.data, data);
//
//         let data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
//         let hash = Hash8::try_from("0102030405060708").unwrap();
//         assert_eq!(hash.data, data);
//     }
//
//     #[test]
//     fn errors_on_invalid_input() {
//         assert!(Hash256::try_from("01").is_err());
//         assert!(Hash8::try_from("01111111111111111111111111111111111111").is_err());
//     }
// }
