//! # Multilayered Linked Spontaneous Ad-Hoc Group Signatures
//! This implementation aims to follow the RingCT whitepaper with certain changes to variables
//! for clarity
use generic_matrix::Matrix;
use rand::rngs::OsRng;

use crypto::{
    CNFastHash,
    Digest,
    ecc::{
        BASEPOINT,
        CompressedPoint,
        Point,
        Scalar
    },
    KeyImage,
    SecretKey
};

type Signature = (Vec<KeyImage>, Scalar, Matrix<Scalar>);

/// SIGN algorithm as defined in the RingCT paper
pub fn sign(message: SecretKey, ring: &Matrix<CompressedPoint>, index: usize, signer_keys: Vec<SecretKey>) -> Result<Signature, failure::Error> {
    // Assertions to ensure input sanity
    // NOTE: KeyMatrix rows contain key vectors, whose columns contain keys
    let rows = ring.row();
    if rows < 2      { return Err(format_err!("Ring must contain more than 1 member")); }
    if index >= rows { return Err(format_err!("Index of signer is outside ring length")); }

    let cols = ring.column();
    if signer_keys.len() != cols { return Err(format_err!("Signer key vector is not consistent with ring matrix")); }

    // Generate key images
    let key_images: Vec<Point> = signer_keys.iter().enumerate().map(|(i_key, x)| {
        x * crypto::ecc::data_to_point(&ring[(index, i_key)])
    }).collect();

    // Generate random scalar vector and matrix for signature
    let alpha: Vec<Scalar> = (0..cols).map(|_| Scalar::random(&mut OsRng::new().unwrap())).collect();
    let mut signature = Matrix::from_fn(rows, cols, |_,_| Scalar::random(&mut OsRng::new().unwrap()));

    // Initialize vectors and matrices
    let mut mat_l = Matrix::from_fn(rows, cols, |i_key_vector, i_key| {
        if i_key_vector == index {
            alpha[i_key] * BASEPOINT
        } else {
            Point::default()
        }
    });
    let mut mat_r = Matrix::from_fn(rows, cols, |i_key_vector, i_key| {
        if i_key_vector == index {
            alpha[i_key] * crypto::ecc::data_to_point(&ring[(index, i_key)])
        } else {
            Point::default()
        }
    });
    let mut vec_c: Vec<Scalar> = (0..rows).map(|_| Scalar::one()).collect();

    // Run hash function
    let mut hasher = CNFastHash::new();
    hasher.input(&bincode::serialize(&message).unwrap());
    for i_key in 0..cols {
        hasher.input(&bincode::serialize(&mat_l[(index, i_key)]).unwrap());
        hasher.input(&bincode::serialize(&mat_r[(index, i_key)]).unwrap());
    }
    vec_c[(index + 1) % rows] = crypto::ecc::hash_to_scalar(hasher.result());

    // Progress the calculation
    for i_key_vector in 1..rows {
        let i_key_vector = (index + i_key_vector) % rows;

        for i_key in 0..cols {
            // L_j = s_j * G + c_j * P_j
            mat_l[(i_key_vector, i_key)] = (signature[(i_key_vector, i_key)] * BASEPOINT) + (vec_c[i_key_vector] * ring[(i_key_vector, i_key)].decompress().unwrap());

            // R_j = s_j * H(P_j) + c_j * I
            mat_r[(i_key_vector, i_key)] = (signature[(i_key_vector, i_key)] * crypto::ecc::data_to_point(&ring[(i_key_vector, i_key)])) + (vec_c[i_key_vector] * key_images[i_key]);
        }

        // c
        let mut hasher = CNFastHash::new();
        hasher.input(&bincode::serialize(&message).unwrap());
        for i_key in 0..cols {
            hasher.input(&bincode::serialize(&mat_l[(i_key_vector, i_key)]).unwrap());
            hasher.input(&bincode::serialize(&mat_r[(i_key_vector, i_key)]).unwrap());
        }

        vec_c[(i_key_vector + 1) % rows] = crypto::ecc::hash_to_scalar(hasher.result());
    }

    // Tweak signature for successful validation
    for (i_key, a) in alpha.iter().enumerate() {
        signature[(index, i_key)] = a - (vec_c[index] * signer_keys[i_key]);
    }

    let key_images = key_images.iter().map(|x| x.compress()).collect();
    Ok((key_images, vec_c[0], signature))
}

/// VERIFY algorithm as defined in the RingCT paper
pub fn verify(message: SecretKey, ring: &Matrix<CompressedPoint>, signature: Signature) -> Result<bool, failure::Error> {
    // Assertions for input sanity
    let rows = ring.row();
    if rows < 2 { return Err(format_err!("Ring must contain more than 1 member")); }

    let cols = ring.column();

    // Initialize matrices and vectors
    let mut mat_l = Matrix::from_fn(rows, cols, |_,_| Point::default());
    let mut mat_r = Matrix::from_fn(rows, cols, |_,_| Point::default());
    let mut c: Vec<Scalar> = (0..=rows).map(|_| Scalar::one()).collect();

    let (key_images, c_0, signature) = signature;

    // Start the chain of computations
    c[0] = c_0;
    for i_key_vector in 0..rows {
        for i_key in 0..cols {
            // L_j = s_j * G + c_j * P_j
            mat_l[(i_key_vector, i_key)] = (signature[(i_key_vector, i_key)] * BASEPOINT) + (c[i_key_vector] * ring[(i_key_vector, i_key)].decompress().unwrap());

            // R_j = s_j * H(P_j) + c_j * I
            mat_r[(i_key_vector, i_key)] = (signature[(i_key_vector, i_key)] * crypto::ecc::data_to_point(&ring[(i_key_vector, i_key)])) + (c[i_key_vector] * key_images[i_key].decompress().unwrap());
        }

        let mut hasher = CNFastHash::new();
        hasher.input(&bincode::serialize(&message).unwrap());
        for i_key in 0..cols {
            hasher.input(&bincode::serialize(&mat_l[(i_key_vector, i_key)]).unwrap());
            hasher.input(&bincode::serialize(&mat_r[(i_key_vector, i_key)]).unwrap());
        }
        c[i_key_vector + 1] = crypto::ecc::hash_to_scalar(hasher.result());
    }

    Ok(c[rows] == c[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let index = 0;
        let keypairs: Vec<crypto::KeyPair> = (0..2).map(|_| crypto::KeyPair::generate()).collect();

        let ring = Matrix::from_fn(3, 2, |i_key_vector, i_key| {
            if i_key_vector == index {
                keypairs[i_key].public_key
            } else {
                crypto::KeyPair::generate().public_key
            }
        });

        let sig_keys = keypairs.iter().map(|x| x.secret_key).collect();
        let message = crypto::KeyPair::generate().secret_key;
        let signature = sign(message, &ring, index, sig_keys);

        assert!(signature.is_ok());
        let signature = signature.unwrap();

        let res = verify(message, &ring, signature);
        assert!(res.is_ok());
        assert!(res.unwrap());
    }

    #[test]
    fn it_errors_on_single_member_rings() {
        let ring = Matrix::from_fn(1, 3, |_,_| crypto::KeyPair::generate().public_key);
        let sig_keys = (0..3).map(|_| crypto::KeyPair::generate().secret_key).collect();
        assert!(sign(crypto::KeyPair::generate().secret_key, &ring, 0, sig_keys).is_err());
    }

    #[test]
    fn it_handles_out_of_bounds_index() {
        let ring = Matrix::from_fn(2, 2, |_,_| crypto::KeyPair::generate().public_key);
        let sig_keys = (0..2).map(|_| crypto::KeyPair::generate().secret_key).collect();
        assert!(sign(crypto::KeyPair::generate().secret_key, &ring, 2, sig_keys).is_err());
    }

    #[test]
    fn it_handles_inconsistent_signer_key_vectors() {
        let ring = Matrix::from_fn(3, 3, |_,_| crypto::KeyPair::generate().public_key);
        let sig_keys = (0..2).map(|_| crypto::KeyPair::generate().secret_key).collect();
        assert!(sign(crypto::KeyPair::generate().secret_key, &ring, 0, sig_keys).is_err());
    }
}
