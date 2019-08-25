use byteorder::ByteOrder;

use crypto::Hash256Data;

pub fn check_hash_for_difficulty(hash: &Hash256Data, difficulty: u128) -> bool {
    let _hash_lo = byteorder::LittleEndian::read_u128(hash);
    let hash_hi = byteorder::LittleEndian::read_u128(&hash[16..]);

    // Check higher half for overflow as most random hashes will fail
    let (_, will_carry) = hash_hi.overflowing_mul(difficulty);
    if will_carry {
        return false;
    }

    // TODO: Do the actual multiplication once widening_mul is stabilized
    // If it overflows, it's not a valid hash
    !will_carry
}
