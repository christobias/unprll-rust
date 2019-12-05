//! # Ring Confidential Transactions (RingCT)
//! This implementation is based on the whitepaper

#[macro_use] extern crate failure;

pub type Matrix<T> = Vec<Vec<T>>;

pub trait MatrixExt<T> {
    fn from_fn(rows: usize, cols: usize, closure: impl Fn(usize, usize) -> T) -> Matrix<T> {
        (0..rows).map(|row| {
            (0..cols).map(|col| closure(row, col)).collect()
        }).collect()
    }
}

impl<T> MatrixExt<T> for Matrix<T> { }

pub mod lsag;
pub mod mlsag;
