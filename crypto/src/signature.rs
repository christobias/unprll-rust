use serde::{Serialize, Deserialize};

use curve25519_dalek::scalar::Scalar;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Signature {
    pub c: Scalar,
    pub r: Scalar
}
