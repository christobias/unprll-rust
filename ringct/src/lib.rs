//! # Ring Confidential Transactions (RingCT)
//! This implementation is based on the whitepaper

#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

use crypto::{
    curve25519_dalek::edwards::EdwardsBasepointTable,
    ecc::{
        CompressedPoint,
        Point
    }
};

pub type Matrix<T> = Vec<Vec<T>>;

pub trait MatrixExt<T> {
    fn from_fn(rows: usize, cols: usize, closure: impl Fn(usize, usize) -> T) -> Matrix<T> {
        (0..rows).map(|row| {
            (0..cols).map(|col| closure(row, col)).collect()
        }).collect()
    }
}

impl<T> MatrixExt<T> for Matrix<T> { }

lazy_static! {
    /// Mask basepoint `H`
    ///
    /// Effectively `to_point(cn_fast_hash(G))` where `G` is our basepoint
    // TODO: Figure out if hardcoding this can be avoided to figure out how it's computed
    pub static ref MASK_BASEPOINT: Point = CompressedPoint::from_slice(
        &[0x8b, 0x65, 0x59, 0x70, 0x15, 0x37, 0x99, 0xaf,
          0x2a, 0xea, 0xdc, 0x9f, 0xf1, 0xad, 0xd0, 0xea,
          0x6c, 0x72, 0x51, 0xd5, 0x41, 0x54, 0xcf, 0xa9,
          0x2c, 0x17, 0x3a, 0x0d, 0xd3, 0x9c, 0x1f, 0x94]
    ).decompress().unwrap();

    /// Mask basepoint `H` in `EdwardsBasepointTable` form
    pub static ref MASK_BASEPOINT_TABLE: EdwardsBasepointTable = EdwardsBasepointTable::create(&MASK_BASEPOINT);
}

pub mod bulletproof;
pub mod mlsag;
