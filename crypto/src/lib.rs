#![deny(missing_docs)]
//! Cryptographic primitives used in Cryptonote

extern crate clear_on_drop;
pub extern crate curve25519_dalek;
extern crate digest;
extern crate rand;

mod cast_256;
mod hash;
mod keys;
mod rnjc;
mod tree_hash;

/// Common elliptic curve cryptography (ECC) operations
pub mod ecc;
pub use digest::Digest;

pub use ecc::ScalarExt;
pub use hash::{CNFastHash, Hash256, Hash256Data};
pub use keys::{KeyImage, KeyPair, PublicKey, SecretKey};
pub use rnjc::RNJC;
pub use tree_hash::tree_hash;
