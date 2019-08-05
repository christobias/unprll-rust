extern crate clear_on_drop;
extern crate curve25519_dalek;
extern crate digest;
extern crate rand;

pub mod cast_256;
mod hash;
mod keys;
mod rnjc;
mod signature;
mod tree_hash;

pub mod ecc {
    pub use curve25519_dalek::scalar::Scalar;
    pub use curve25519_dalek::edwards::CompressedEdwardsY as CompressedPoint;
    pub use curve25519_dalek::edwards::EdwardsPoint as Point;
    pub use curve25519_dalek::constants::ED25519_BASEPOINT_POINT as BASEPOINT;

    use super::Digest;
    use super::CNFastHash;

    pub fn hash_to_scalar(hash: crate::hash::Hash256Data) -> Scalar {
        let mut buf: [u8; 32] = [0; 32];
        buf.copy_from_slice(&hash);
        Scalar::from_bytes_mod_order(buf)
    }

    pub fn data_to_scalar<T: serde::Serialize>(data: &T) -> Scalar {
        let hash = CNFastHash::digest(&bincode::serialize(&data).unwrap());
        hash_to_scalar(hash)
    }

    pub fn data_to_point<T: serde::Serialize>(data: &T) -> Point {
        data_to_scalar(data) * BASEPOINT
    }
}

pub use digest::Digest;

pub use hash::{Hash256,Hash256Data,CNFastHash};
pub use keys::{SecretKey,PublicKey,KeyPair,KeyImage};
pub use rnjc::RNJC;
pub use signature::Signature;
pub use tree_hash::tree_hash;
