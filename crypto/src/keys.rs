use crate::ecc::{
    Scalar,
    CompressedPoint,
    BASEPOINT
};
use rand;
use serde::{
    Serialize,
    Deserialize
};

/// An unsigned 256-bit value used as a private key. Represented with lowercase letters
pub type SecretKey = Scalar;

/// A point on the elliptic curve. Usually determined by multiplication of a scalar to the curve
/// basepoint
pub type PublicKey = CompressedPoint;

/// Type alias specific to Cryptonote
pub type KeyImage = PublicKey;

/// Helper Extension Trait for Scalar
pub trait ScalarExt {
    /// Generates a Scalar from a [u8] slice
    ///
    /// The constructor for Scalar requires a [u8; 32] (for obvious reasons)
    /// However, the code for converting between a slice and [u8; 32] tends to be repeated,
    /// hence, this common implementation
    fn from_slice(data: &[u8]) -> Scalar {
        let mut scalar: [u8; 32] = [0; 32];
        scalar.copy_from_slice(data);
        Scalar::from_bytes_mod_order(scalar)
    }
}

impl ScalarExt for Scalar { }

/// A pair of a given secret key and its corresponding public key
#[derive(Serialize, Deserialize)]
pub struct KeyPair {
    /// The secret key
    pub secret_key: SecretKey,
    /// The public key
    pub public_key: PublicKey
}

impl KeyPair {
    /// Generates a random keypair using the OS CSPRNG
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng::new().unwrap();
        let secret_key = Scalar::random(&mut rng);

        Self::from(secret_key)
    }
}

impl From<Scalar> for KeyPair {
    fn from(secret_key: SecretKey) -> Self {
        let public_key = (&secret_key * &BASEPOINT).compress();
        Self {
            secret_key,
            public_key
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Unprll donation wallet viewkey
        let kp = KeyPair::from(Scalar::from_slice(&hex::decode("cae2b02f3a317b0ef61e694d899060f8434aef556bfe60239846533b52ab4608").unwrap()));
        assert_eq!(hex::encode(kp.public_key.as_bytes()), "36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0");
    }
}
