use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT,
    edwards::CompressedEdwardsY,
    scalar::Scalar
};
use rand;

pub type SecretKey = Scalar;
pub type PublicKey = CompressedEdwardsY;
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

pub struct KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey
}

impl KeyPair {
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng::new().unwrap();
        let secret_key = Scalar::random(&mut rng);

        Self::from(secret_key)
    }
}

impl From<Scalar> for KeyPair {
    fn from(secret_key: Scalar) -> Self {
        // TODO: Find out why basepoint table scalar multiplication doesn't work
        let public_key = (secret_key * ED25519_BASEPOINT_POINT).compress();
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
