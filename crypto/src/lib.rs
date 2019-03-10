#[macro_use] extern crate arrayref;
extern crate clear_on_drop;
extern crate curve25519_dalek;
extern crate rand;

pub mod cast_256;
pub mod hash;
pub mod keys;
pub mod rnjc;
pub mod tree_hash;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
