//! # Linkable Spontaneous Anonymous Group Signatures
//! This implementation aims to follow the RingCT whitepaper with certain changes to variables
//! for clarity
//!
//! **NOTE:** LSAGs are not used in RingCT (the more generic MLSAGs are used). This is a reference implementation 
use crypto::{
    CNFastHash,
    Digest,
    ecc::{
        BASEPOINT,
        Point,
        Scalar
    },
    KeyImage,
    SecretKey,
    PublicKey
};

type Signature = (KeyImage, Scalar, Vec<Scalar>);

pub fn sign(message: SecretKey, ring: &Vec<PublicKey>, index: usize, signing_key: SecretKey) -> Signature {
    let len = ring.len();
    // Generate key image
    let key_image = signing_key * crypto::ecc::data_to_point(&ring[index]);

    let mut csprng = rand::rngs::OsRng::new().unwrap();

    // Generate random scalars for signature
    let alpha = Scalar::random(&mut csprng);
    let mut s: Vec<Scalar> = (0..len).map(|_| Scalar::random(&mut csprng)).collect();

    // Initialize L, R and c vectors
    let mut vec_l: Vec<Point> = (0..len).map(|_| Point::default()).collect();
    let mut vec_r = vec_l.clone();
    let mut vec_c: Vec<Scalar> = (0..len).map(|_| Scalar::one()).collect();

    // Start at given secret index
    vec_l[index] = alpha * BASEPOINT;
    vec_r[index] = alpha * crypto::ecc::data_to_point(&ring[index]);

    // Run hash function
    let mut hasher = CNFastHash::new();
    hasher.input(&bincode::serialize(&message).unwrap());
    hasher.input(&bincode::serialize(&vec_l[index]).unwrap());
    hasher.input(&bincode::serialize(&vec_r[index]).unwrap());
    vec_c[(index + 1) % len] = crypto::ecc::hash_to_scalar(hasher.result());

    // Progress the calculation
    for i in 1..len {
        let i = (index + i) % len;
        vec_l[i] = s[i] * BASEPOINT + vec_c[i] * ring[i].decompress().unwrap();
        vec_r[i] = s[i] * crypto::ecc::data_to_point(&ring[i]) + vec_c[i] * key_image;

        let mut hasher = CNFastHash::new();
        hasher.input(&bincode::serialize(&message).unwrap());
        hasher.input(&bincode::serialize(&vec_l[i]).unwrap());
        hasher.input(&bincode::serialize(&vec_r[i]).unwrap());
        vec_c[(i + 1) % len] = crypto::ecc::hash_to_scalar(hasher.result());
    }

    // Tweak signature for successful validation
    s[index] = alpha - (vec_c[index] * signing_key);

    (key_image.compress(), vec_c[0], s)
}

pub fn verify(message: SecretKey, ring: &Vec<PublicKey>, signature: Signature) -> bool {
    let len = ring.len();
    let (key_image, c_0, s) = signature;

    let mut vec_l: Vec<Point> = (0..len).map(|_| Point::default()).collect();
    let mut vec_r = vec_l.clone();
    let mut vec_c: Vec<Scalar> = (0..(len + 1)).map(|_| Scalar::default()).collect();

    vec_c[0] = c_0;

    for i in 0..len {
        vec_l[i] = s[i] * BASEPOINT + vec_c[i] * ring[i].decompress().unwrap();
        vec_r[i] = s[i] * crypto::ecc::data_to_point(&ring[i]) + vec_c[i] * key_image.decompress().unwrap();

        let mut hasher = CNFastHash::new();
        hasher.input(&bincode::serialize(&message).unwrap());
        hasher.input(&bincode::serialize(&vec_l[i]).unwrap());
        hasher.input(&bincode::serialize(&vec_r[i]).unwrap());
        vec_c[i + 1] = crypto::ecc::hash_to_scalar(hasher.result());
    }


    c_0 == vec_c[len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let index = 1;
        let keypair = crypto::KeyPair::generate();

        let mut ring = Vec::new();
        for _ in 0..2 {
            ring.push(crypto::KeyPair::generate().public_key);
        }
        ring[index] = keypair.public_key;

        let sig_keys = keypair.secret_key;
        let message = crypto::KeyPair::generate().secret_key;
        let signature = sign(message, &ring, index, sig_keys);

        let res = verify(message, &ring, signature);
        assert!(res);
    }
}
