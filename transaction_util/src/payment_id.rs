//! Module for encrypting and decrypting payment IDs

use crypto::{ecc::Point, CNFastHash, Digest, Hash8, Hash8Data};

/// Encrypts a payment ID
/// 
/// Encryption is done by taking a hash of a shared key derivation and
/// bitwise XOR'ing it with the payment ID
pub fn encrypt(payment_id: Hash8, key_derivation: Point) -> Hash8 {
    let mut hasher = CNFastHash::new();

    hasher.input(key_derivation.compress().to_bytes());
    hasher.input(&[0x8d]);

    let hash = hasher.result();

    Hash8::from(
        payment_id
            .data()
            .iter()
            .zip(hash.iter())
            .map(|(pid, hash)| pid ^ hash)
            .collect::<Hash8Data>(),
    )
}