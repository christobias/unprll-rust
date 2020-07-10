use crypto::{
    ecc::{Point, Scalar},
    CNFastHash, Digest, KeyPair, PublicKey, ScalarExt,
};

/// Wrapper around the result (secret key * public key)
pub struct Derivation(pub(crate) Point);

impl Derivation {
    /// Create a new derivation from the given secret and public keys
    pub fn from(scalar: &Scalar, public_key: &PublicKey) -> Option<Self> {
        if !scalar.is_canonical() {
            return None;
        }

        Some(Derivation((scalar * public_key).mul_by_cofactor()))
    }

    /// Convert this derivation into a Scalar
    /// H_s(derivation || output_index)
    pub fn to_scalar(&self, output_index: u64) -> Scalar {
        let mut hasher = CNFastHash::new();

        hasher.input(self.0.compress().as_bytes());
        hasher.input(varint::serialize(output_index));

        Scalar::from_slice(&hasher.result())
    }

    /// Convert this derivation into a KeyPair with the following keys
    ///
    /// * Secret d: H_s(derivation || output_index)
    /// * Public: dG + mask_point
    pub fn to_keypair(&self, output_index: u64, mask_point: Point) -> KeyPair {
        let mut keypair = KeyPair::from(self.to_scalar(output_index));
        keypair.public_key += mask_point;

        keypair
    }
}
