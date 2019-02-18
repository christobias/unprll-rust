#[macro_use] extern crate arrayref;

pub mod cast_256;
pub mod hash;
pub mod rnjc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
