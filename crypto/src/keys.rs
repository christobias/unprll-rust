use curve25519_dalek::{scalar::Scalar, edwards::CompressedEdwardsY, constants::ED25519_BASEPOINT_POINT};
use rand;

use crate::hash::Hash256;

pub type SecretKey = Scalar;
pub type PublicKey = CompressedEdwardsY;
pub type KeyImage = PublicKey;

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
        let public_key = (secret_key * &ED25519_BASEPOINT_POINT).compress();
        let kp = Self {
            secret_key,
            public_key
        };
        kp
    }
}

impl From<Hash256> for KeyPair {
    fn from(secret_key: Hash256) -> Self {
        let mut scalar: [u8; 32] = [0; 32];
        scalar.copy_from_slice(&secret_key.data());
        let secret_key = Scalar::from_bytes_mod_order(scalar);

        Self::from(secret_key)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use super::*;

    #[test]
    fn it_works() {
        // Unprll donation wallet viewkey
        let kp = KeyPair::from(Hash256::try_from("cae2b02f3a317b0ef61e694d899060f8434aef556bfe60239846533b52ab4608").unwrap());
        assert_eq!(hex::encode(kp.public_key.as_bytes()), "36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0");
    }
}
