use crate::{
    CNFastHash,
    Digest
};

mod from_c {
    use std::os::raw::{
        c_int,
        c_uchar
    };

    #[link(name = "hashtopoint", kind = "static")]
    extern {
        pub fn hash_to_point(data: *const c_uchar, result: *mut c_uchar) -> c_int;
    }
}

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

pub use curve25519_dalek::scalar::Scalar;
pub use curve25519_dalek::edwards::CompressedEdwardsY as CompressedPoint;
pub use curve25519_dalek::edwards::EdwardsPoint as Point;
pub use curve25519_dalek::constants::ED25519_BASEPOINT_POINT as BASEPOINT;
pub use curve25519_dalek::constants::ED25519_BASEPOINT_COMPRESSED as BASEPOINT_COMPRESSED;
pub use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE as BASEPOINT_TABLE;

/// Converts a given hash to a `Scalar`
pub fn hash_to_scalar(hash: crate::hash::Hash256Data) -> Scalar {
    let mut buf: [u8; 32] = [0; 32];
    buf.copy_from_slice(&hash);
    Scalar::from_bytes_mod_order(buf)
}

/// Converts a given hash to a `Point`
/// 
/// Uses ge_fromfe_frombytes_vartime from Monero
pub fn hash_to_point(hash: crate::hash::Hash256Data) -> Point {
    // Double hash
    let hash = CNFastHash::digest(&hash);
    let mut result: [u8; 32] = [0; 32];
    unsafe {
        let ret = from_c::hash_to_point(hash.as_ptr(), result.as_mut_ptr());
        // TODO: Make this more rigorous by having a Result based API
        assert!(ret == 0);
    }

    CompressedPoint::from_slice(&result).decompress().unwrap().mul_by_cofactor()
}