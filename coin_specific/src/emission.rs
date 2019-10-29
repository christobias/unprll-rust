pub const MONEY_SUPPLY: u64 = -1i64 as u64;

pub use cryptonote_core::EmissionCurve;

use crate::Unprll;

impl EmissionCurve for Unprll {
    fn get_block_reward(&self, _version: u8) -> Result<u64, failure::Error> {
        Ok(17_590_000_000_000)
    }
}
