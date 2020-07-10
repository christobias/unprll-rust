//! # Multilayered Linked Spontaneous Ad-Hoc Group Signatures
//! This implementation aims to follow the RingCT whitepaper with certain changes to variables
//! for clarity

// The range loops we use here aren't really unnecessary as we need the index to multiple Vecs
#![allow(clippy::needless_range_loop)]

use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crypto::{
    ecc::{CompressedPoint, Scalar, BASEPOINT_TABLE},
    CNFastHash, Digest, KeyImage, SecretKey,
};

use crate::Matrix;

/// Error type for MLSAG operations
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when the signature parameters are inconsistent
    #[fail(display = "Input parameters are inconsistent")]
    InconsistentParameters,

    /// Returned when the signature is inconsistent
    #[fail(display = "Signature is inconsistent")]
    InconsistentSignature,

    /// Returned when the signature fails to verify correctly
    #[fail(display = "Signature is invalid")]
    InvalidSignature,
}

/// MLSAG signature
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    pub s: Matrix<Scalar>,
    pub c: Scalar,

    /// Key Images
    pub key_images: Vec<KeyImage>,
}

impl Default for Signature {
    fn default() -> Signature {
        Signature {
            s: Matrix::from_fn(0, 0, |_, _| Scalar::default()).unwrap(),
            c: Scalar::default(),
            key_images: Vec::new(),
        }
    }
}

/// SIGN algorithm as defined in Monero
///
/// The version implemented in Monero differs from the version defined in
/// the RingCT whitepaper in that it allows specifying which keys need a key image
///
/// NOTE: The mixin ring uses CompressedPoint as most of MLSAG involves hashing
/// compressed public keys
pub fn sign(
    message: &[u8],
    ring: &Matrix<CompressedPoint>,
    index: u64,
    signer_keys: &[SecretKey],
    double_spendable_keys: u64,
) -> Result<Signature, Error> {
    // Assertions to ensure input sanity
    // NOTE: KeyMatrix rows contain key vectors, whose columns contain keys
    let index = index as usize;
    let double_spendable_keys = double_spendable_keys as usize;

    let rows = ring.rows();
    if rows < 2 {
        return Err(Error::InconsistentParameters);
    }
    if index >= rows {
        return Err(Error::InconsistentParameters);
    }

    let cols = ring.cols();
    if signer_keys.len() != cols {
        return Err(Error::InconsistentParameters);
    }

    // Generate key images
    let key_images: Vec<_> = signer_keys
        .iter()
        .enumerate()
        .zip(0..double_spendable_keys)
        .map(|((i_key, x), _)| {
            x * crypto::ecc::hash_to_point(CNFastHash::digest(ring[(index, i_key)].as_bytes()))
        })
        .collect();

    // Generate random scalar vector and matrix for signature
    let alpha: Vec<Scalar> = (0..cols).map(|_| Scalar::random(&mut OsRng)).collect();
    let mut signature = Matrix::from_fn(rows, cols, |_, _| Scalar::random(&mut OsRng)).unwrap();

    let mut hasher = CNFastHash::new();

    hasher.input(message);
    for i_key in 0..double_spendable_keys {
        hasher.input(ring[(index, i_key)].as_bytes());
        hasher.input((&alpha[i_key] * &BASEPOINT_TABLE).compress().as_bytes());
        hasher.input(
            (alpha[i_key]
                * crypto::ecc::hash_to_point(CNFastHash::digest(ring[(index, i_key)].as_bytes())))
            .compress()
            .as_bytes(),
        );
    }
    for i_key in double_spendable_keys..cols {
        hasher.input(ring[(index, i_key)].as_bytes());
        hasher.input((&alpha[i_key] * &BASEPOINT_TABLE).compress().as_bytes());
    }

    let mut vec_c: Vec<Scalar> = (0..rows).map(|_| Scalar::one()).collect();
    vec_c[(index + 1) % rows] = crypto::ecc::hash_to_scalar(hasher.result_reset());

    // Progress the calculation
    for i_key_vector in 1..rows {
        let i_key_vector = (index + i_key_vector) % rows;

        hasher.input(message);
        for i_key in 0..double_spendable_keys {
            hasher.input(ring[(i_key_vector, i_key)].as_bytes());
            // L_j = s_j * G + c_j * P_j
            hasher.input(
                ((&signature[(i_key_vector, i_key)] * &BASEPOINT_TABLE)
                    + (vec_c[i_key_vector] * ring[(i_key_vector, i_key)].decompress().unwrap()))
                .compress()
                .as_bytes(),
            );

            // R_j = s_j * H(P_j) + c_j * I
            hasher.input(
                ((signature[(i_key_vector, i_key)]
                    * crypto::ecc::hash_to_point(CNFastHash::digest(
                        ring[(i_key_vector, i_key)].as_bytes(),
                    )))
                    + (vec_c[i_key_vector] * key_images[i_key]))
                    .compress()
                    .as_bytes(),
            )
        }

        for i_key in double_spendable_keys..cols {
            hasher.input(ring[(i_key_vector, i_key)].as_bytes());
            // L_j = s_j * G + c_j * P_j
            hasher.input(
                ((&signature[(i_key_vector, i_key)] * &BASEPOINT_TABLE)
                    + (vec_c[i_key_vector] * ring[(i_key_vector, i_key)].decompress().unwrap()))
                .compress()
                .as_bytes(),
            );
        }

        // c
        vec_c[(i_key_vector + 1) % rows] = crypto::ecc::hash_to_scalar(hasher.result_reset());
    }

    // Tweak signature for successful validation
    for (i_key, a) in alpha.iter().enumerate() {
        signature[(index, i_key)] = a - (vec_c[index] * signer_keys[i_key]);
    }

    let s = Signature {
        s: signature,
        c: vec_c[0],
        key_images,
    };

    Ok(s)
}

/// VERIFY algorithm as defined in the RingCT paper
pub fn verify(
    message: &[u8],
    ring: &Matrix<CompressedPoint>,
    signature: &Signature,
    double_spendable_keys: usize,
) -> Result<(), Error> {
    // Assertions for input sanity
    let rows = ring.rows();
    if rows < 2 {
        return Err(Error::InconsistentSignature);
    }

    let cols = ring.cols();
    if cols < 1 {
        return Err(Error::InconsistentSignature);
    }
    if double_spendable_keys > cols {
        return Err(Error::InconsistentSignature);
    }
    if signature.s.rows() != rows || signature.s.cols() != cols {
        return Err(Error::InconsistentSignature);
    }
    if signature.key_images.len() != double_spendable_keys {
        return Err(Error::InconsistentSignature);
    }

    let Signature {
        key_images,
        c: c_0,
        s: signature,
    } = signature;

    // Start the chain of computations
    let mut hasher = CNFastHash::new();

    let mut last_c = *c_0;
    for i_key_vector in 0..rows {
        hasher.input(message);

        // Start with the double spendable keys
        for i_key in 0..double_spendable_keys {
            // L_j = s_j * G + c_j * P_j
            let l = (&signature[(i_key_vector, i_key)] * &BASEPOINT_TABLE)
                + (last_c * ring[(i_key_vector, i_key)].decompress().unwrap());
            // R_j = s_j * H(P_j) + c_j * I
            let r = (signature[(i_key_vector, i_key)]
                * crypto::ecc::hash_to_point(CNFastHash::digest(
                    ring[(i_key_vector, i_key)].as_bytes(),
                )))
                + (last_c * key_images[i_key]);

            // pubkey || L || R
            hasher.input(ring[(i_key_vector, i_key)].as_bytes());
            hasher.input(l.compress().as_bytes());
            hasher.input(r.compress().as_bytes());
        }

        // Continue with the non double spendable keys
        for i_key in double_spendable_keys..cols {
            // L_j = s_j * G + c_j * P_j
            let l = (&signature[(i_key_vector, i_key)] * &BASEPOINT_TABLE)
                + (last_c * ring[(i_key_vector, i_key)].decompress().unwrap());

            // pubkey || L
            hasher.input(ring[(i_key_vector, i_key)].as_bytes());
            hasher.input(l.compress().as_bytes());
        }
        last_c = crypto::ecc::hash_to_scalar(hasher.result_reset());
    }

    if last_c != *c_0 {
        return Err(Error::InvalidSignature);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn it_works() {
        let index = (rand::rngs::OsRng.next_u32() % 3) as usize;
        let keypairs: Vec<crypto::KeyPair> = (0..2).map(|_| crypto::KeyPair::generate()).collect();

        let ring = Matrix::from_fn(3, 2, |i_key_vector, i_key| {
            if i_key_vector == index {
                keypairs[i_key].public_key
            } else {
                crypto::KeyPair::generate().public_key
            }
            .compress()
        })
        .unwrap();

        let sig_keys: Vec<_> = keypairs.iter().map(|x| x.secret_key).collect();
        let message = crypto::KeyPair::generate().secret_key;
        let signature = sign(message.as_bytes(), &ring, index as u64, &sig_keys, 1);

        assert!(signature.is_ok());
        let signature = signature.unwrap();

        let res = verify(message.as_bytes(), &ring, &signature, 1);
        res.unwrap()
    }

    #[test]
    fn it_refuses_single_member_rings() {
        let ring = Matrix::from_fn(1, 3, |_, _| {
            crypto::KeyPair::generate().public_key.compress()
        })
        .unwrap();
        let sig_keys: Vec<_> = (0..3)
            .map(|_| crypto::KeyPair::generate().secret_key)
            .collect();

        let res = sign(
            crypto::KeyPair::generate().secret_key.as_bytes(),
            &ring,
            0,
            &sig_keys,
            3,
        );
        match res.unwrap_err() {
            Error::InconsistentParameters => {}
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn it_handles_out_of_bounds_secret_index() {
        let ring = Matrix::from_fn(2, 2, |_, _| {
            crypto::KeyPair::generate().public_key.compress()
        })
        .unwrap();
        let sig_keys: Vec<_> = (0..2)
            .map(|_| crypto::KeyPair::generate().secret_key)
            .collect();

        let res = sign(
            crypto::KeyPair::generate().secret_key.as_bytes(),
            &ring,
            2,
            &sig_keys,
            2,
        );
        match res.unwrap_err() {
            Error::InconsistentParameters => {}
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn it_handles_inconsistent_signer_key_vectors() {
        let ring = Matrix::from_fn(3, 3, |_, _| {
            crypto::KeyPair::generate().public_key.compress()
        })
        .unwrap();
        let sig_keys: Vec<_> = (0..2)
            .map(|_| crypto::KeyPair::generate().secret_key)
            .collect();

        let res = sign(
            crypto::KeyPair::generate().secret_key.as_bytes(),
            &ring,
            0,
            &sig_keys,
            3,
        );
        match res.unwrap_err() {
            Error::InconsistentParameters => {}
            _ => panic!("Wrong error type"),
        }
    }
}
