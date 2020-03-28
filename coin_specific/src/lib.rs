use serde::{Deserialize, Serialize};

pub mod emission;

pub const COIN_NAME: (&str, &str) = ("Unprll", "ULL");
pub const VERSION: (&str, &str) = ("v1.0.0", "Rusty Rabbit");

#[derive(Serialize, Deserialize)]
pub struct Unprll;

impl wallet::AddressPrefixes for Unprll {
    const STANDARD: u64 = 0x0014_5023; // UNP
    const SUBADDRESS: u64 = 0x0021_1023; // UNPS
    const INTEGRATED: u64 = 0x0029_1023; // UNPi
}
