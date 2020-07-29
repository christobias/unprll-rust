#![deny(missing_docs)]

//! # Ring Confidential Transactions (RingCT)
//! This implementation is based on the whitepaper

#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;

use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crypto::{
    curve25519_dalek::{edwards::EdwardsBasepointTable, traits::MultiscalarMul},
    ecc::{CompressedPoint, Point, Scalar, BASEPOINT},
};

lazy_static! {
    /// Mask basepoint `H`
    ///
    /// Effectively `to_point(cn_fast_hash(G))` where `G` is our basepoint
    // TODO: Figure out if hardcoding this can be avoided to figure out how it's computed
    pub static ref AMOUNT_BASEPOINT: Point = CompressedPoint::from_slice(
        &[0x8b, 0x65, 0x59, 0x70, 0x15, 0x37, 0x99, 0xaf,
          0x2a, 0xea, 0xdc, 0x9f, 0xf1, 0xad, 0xd0, 0xea,
          0x6c, 0x72, 0x51, 0xd5, 0x41, 0x54, 0xcf, 0xa9,
          0x2c, 0x17, 0x3a, 0x0d, 0xd3, 0x9c, 0x1f, 0x94]
    ).decompress().unwrap();

    /// Mask basepoint `H` in `EdwardsBasepointTable` form
    pub static ref AMOUNT_BASEPOINT_TABLE: EdwardsBasepointTable = EdwardsBasepointTable::create(&AMOUNT_BASEPOINT);
}

/// Pedersen Commitments
///
/// `C = aG + bH`
#[derive(Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// The value being committed to `b`
    pub value: Scalar,
    /// The blinding factor `a`
    pub mask: Scalar,
}

impl Commitment {
    /// Generate a commitment to the given value using a random mask
    pub fn commit_to_value(value: u64) -> Commitment {
        Commitment {
            value: Scalar::from(value),
            mask: Scalar::random(&mut rand::rngs::OsRng),
        }
    }

    /// Returns the result of the commitment
    ///
    /// Computes `C` where `C = aG + bH`
    pub fn into_public(self) -> Point {
        Point::multiscalar_mul(&[self.mask, self.value], &[BASEPOINT, *AMOUNT_BASEPOINT])
    }

    /// Returns the result of the commitment
    ///
    /// Computes `C` where `C = aG + bH`
    pub fn as_public(&self) -> Point {
        Point::multiscalar_mul(&[self.mask, self.value], &[BASEPOINT, *AMOUNT_BASEPOINT])
    }
}

/// Non-zero 2D array of data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Matrix<T>(Vec<Vec<T>>);

impl<T> Matrix<T> {
    /// Create a new Matrix from a function
    ///
    /// The closure is called with the current row and column as parameters
    pub fn from_fn(
        rows: usize,
        cols: usize,
        closure: impl Fn(usize, usize) -> T,
    ) -> Option<Matrix<T>> {
        if rows == 0 || cols == 0 {
            return None;
        }

        Some(Matrix(
            (0..rows)
                .map(|row| (0..cols).map(|col| closure(row, col)).collect())
                .collect(),
        ))
    }

    /// Create a new Matrix from an iterator
    ///
    /// The given iterator is a one dimensional version of the matrix in row major order
    pub fn from_iter(
        rows: usize,
        cols: usize,
        iter: impl IntoIterator<Item = T>,
    ) -> Option<Matrix<T>> {
        if rows == 0 || cols == 0 {
            return None;
        }

        let mut iter = iter.into_iter();
        let m = Matrix(
            (0..rows)
                .map(|_| (0..cols).map(|_| iter.next()).collect::<Option<_>>())
                .collect::<Option<_>>()?,
        );
        if iter.next().is_some() {
            None
        } else {
            Some(m)
        }
    }

    /// Get the number of rows in this matrix
    pub fn rows(&self) -> usize {
        self.0.len()
    }

    /// Get the number of columns in this matrix
    pub fn cols(&self) -> usize {
        self.0[0].len()
    }

    /// Get an iterator over each row
    pub fn row_iter(&self) -> impl Iterator<Item = &Vec<T>> {
        self.0.iter()
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &T {
        &self.0[row][col]
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut T {
        &mut self.0[row][col]
    }
}

pub mod bulletproof;
pub mod mlsag;
mod ringct;

pub use crate::ringct::{
    decode, sign, verify_multiple, DestinationCommitmentPair, Error, RingCTBase, RingCTInput, RingCTOutput,
    RingCTSignature, RingCTType,
};
