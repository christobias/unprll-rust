#![deny(missing_docs)]

//! Bincode is a crate for encoding and decoding using a tiny binary
//! serialization strategy.  Using it, you can easily go from having
//! an object in memory, quickly serialize it to bytes, and then
//! deserialize it back just as fast!
//!
//! ### Using Basic Functions
//!
//! ```edition2018
//! fn main() {
//!     // The object that we will serialize.
//!     let target: Option<String>  = Some("hello world".to_string());
//!
//!     let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
//!     let decoded: Option<String> = bincode::deserialize(&encoded[..]).unwrap();
//!     assert_eq!(target, decoded);
//! }
//! ```
//!
//! ### 128bit numbers
//!
//! Support for `i128` and `u128` is automatically enabled on Rust toolchains
//! greater than or equal to `1.26.0`.

#![crate_name = "bincode_epee"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

extern crate byteorder;
#[macro_use]
extern crate serde;

mod config;
mod error;
mod internal;
mod ser;

pub use config::Config;
pub use error::{Error, ErrorKind, Result};

/// An object that implements this trait can be passed a
/// serde::Serializer without knowing its concrete type.
///
/// This trait should be used only for `with_serializer` functions.
#[doc(hidden)]
pub trait SerializerAcceptor {
    /// The return type for the accept method
    type Output;
    /// Accept a serde::Serializer and do whatever you want with it.
    fn accept<T: serde::Serializer>(self, T) -> Self::Output;
}

/// Get a default configuration object.
///
/// ### Default Configuration:
///
/// | Byte limit | Endianness |
/// |------------|------------|
/// | Unlimited  | Little     |
#[inline(always)]
pub fn config() -> Config {
    Config::new()
}

/// Serializes an object directly into a `Writer` using the default configuration.
///
/// If the serialization would take more bytes than allowed by the size limit, an error
/// is returned and *no bytes* will be written into the `Writer`.
pub fn serialize_into<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::Serialize,
{
    config().serialize_into(writer, value)
}

/// Serializes a serializable object into a `Vec` of bytes using the default configuration.
pub fn serialize<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    config().serialize(value)
}

/// Returns the size that an object would be if serialized using Bincode with the default configuration.
pub fn serialized_size<T: ?Sized>(value: &T) -> Result<u64>
where
    T: serde::Serialize,
{
    config().serialized_size(value)
}

/// Executes the acceptor with a serde::Serializer instance.
/// NOT A PART OF THE STABLE PUBLIC API
#[doc(hidden)]
pub fn with_serializer<A, W>(writer: W, acceptor: A) -> A::Output
where
    A: SerializerAcceptor,
    W: std::io::Write,
{
    config().with_serializer(writer, acceptor)
}
