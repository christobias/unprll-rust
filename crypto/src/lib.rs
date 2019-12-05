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
mod signature;
mod tree_hash;

/// Common elliptic curve cryptography (ECC) operations
pub mod ecc;
pub use digest::Digest;

pub use ecc::ScalarExt;
pub use hash::{Hash256,Hash256Data,CNFastHash};
pub use keys::{SecretKey,PublicKey,KeyPair,KeyImage};
pub use rnjc::RNJC;
pub use signature::Signature;
pub use tree_hash::tree_hash;
