use serde::{Deserialize, Serialize};

use curve25519_dalek::scalar::Scalar;

/// A pre-RingCT signature
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Signature {
    // TODO: Get better names and document them
    #[allow(missing_docs)]
    pub c: Scalar,
    #[allow(missing_docs)]
    pub r: Scalar,
}
