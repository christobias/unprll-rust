#![cfg(test)]

pub struct TestCoin;

impl crate::address::AddressPrefixes for TestCoin {
    const STANDARD:   u64 = 0x0014_5023; // UNP
    const SUBADDRESS: u64 = 0x0021_1023; // UNPS
    const INTEGRATED: u64 = 0x0029_1023; // UNPi
}
