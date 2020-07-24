/// This macro is based off the `ensure!` macro in dtolnay's anyhow crate,
/// but made to return `std::error::Error` instead of `anyhow::Error`
///
/// A macro similar to `assert!` but returns an `std::error::Error` instead of panicking
///
/// This macro is equivalent to `if !$cond { return Err($err); }`.
///
/// ```
/// # use ensure_macro::ensure;
/// # #[derive(Debug)]
/// enum ExampleError {
///     Example
/// }
/// #
/// # impl std::fmt::Display for ExampleError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #         write!(f, "Example Error");
/// #         Ok(())
/// #     }
/// # }
///
/// impl std::error::Error for ExampleError { }
///
/// # fn main() -> Result<(), ExampleError> {
/// ensure!((1 < 2), ExampleError::Example);
/// #    Ok(())
/// # }
///
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}
