use byteorder::ByteOrder;

use crypto::Hash256Data;

/// Wide u128 Multiplication
/// Returns the high and low bytes from a u128 * u128 multiplication
/// To be replaced with a standard widening_mul
///
/// # Returns
/// A tuple (low, high) where low is the low word and high is the high word
fn widening_mul(a: u128, b: u128) -> (u128, u128) {
    const U64_MASK: u128 = (1 << 64) - 1;

    // Get the low and high halfs. Each 128 bit value below contains a 64 vit value
    let a_lo = a & U64_MASK;
    let a_hi = a >> 64;
    let b_lo = b & U64_MASK;
    let b_hi = b >> 64;

    // lo * lo
    let res = a_lo * b_lo;
    let res_1_lo = res & U64_MASK;
    let carry = res >> 64;

    // hi * lo + carry
    let res = a_hi * b_lo + carry;
    let res_1_hi = res & U64_MASK;
    let res_1_of = res >> 64;

    // lo * hi
    let res = a_lo * b_hi;
    let res_2_lo = res & U64_MASK;
    let carry = res >> 64;

    // hi * hi + carry
    let res = a_hi * b_hi + carry;
    let res_2_hi = res & U64_MASK;
    let res_2_of = res >> 64;

    //  (high              , low             )
    //            res_1_of | res_1_hi res_1_lo
    // + res_2_of res_2_hi | res_2_lo

    // res_1_hi + res_2_lo
    let res = res_1_hi + res_2_lo;
    let carry = res >> 64;

    // Final low word
    let low = res << 64 | res_1_lo;

    // res_1_of + res_2_hi
    let res = res_1_of + res_2_hi + carry;
    let carry = res >> 64;

    // Final high word
    let high = ((res_2_of + carry) << 64) | res;

    (low, high)
}

/// Checks a given hash for a certain difficulty
///
/// A given hash is valid for a certain difficulty if the relation `hash * difficulty <= 2.pow(256)`
/// is true. In other words, the product of hash and difficulty must fit without overflow into a
/// 256-bit integer. The hash is interpreted as a little-endian 256-bit value
pub fn check_hash_for_difficulty(hash: &Hash256Data, difficulty: u128) -> bool {
    let hash_lo = byteorder::LittleEndian::read_u128(hash);
    let hash_hi = byteorder::LittleEndian::read_u128(&hash[16..]);

    // Check higher half for overflow as most random hashes will fail
    let (_, will_carry) = hash_hi.overflowing_mul(difficulty);
    if will_carry {
        return false;
    }

    // Multiply low half
    let (_, carry_lo) = widening_mul(hash_lo, difficulty);
    // Multiply high half with carry
    let (res_hi, carry) = widening_mul(hash_hi, difficulty);

    // If it overflows, it's not a valid hash
    if carry == 0 {
        let (_, will_carry) = res_hi.overflowing_add(carry_lo);
        !will_carry
    } else {
        // Overflow before carry addition
        false
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crypto::Hash256;

    use super::*;

    #[test]
    fn widening_mul_works() {
        [
            // Multiply by 0
            (0, 0, (0, 0)),
            (std::u128::MAX, 0, (0, 0)),
            // Multiply by 1
            (20, 1, (20, 0)),
            // Multiply some numbers (nothing really special about these numbers, just some keyboard mashing)
            (
                943_850_348_584_379,
                547_653_733_455_224,
                (516_903_167_225_249_755_920_782_345_896, 0),
            ),
            // Handle u128 overflow
            (std::u128::MAX, 2, (std::u128::MAX - 1, 1)),
            // Stress test (maximum possible result)
            (std::u128::MAX, std::u128::MAX, (1, std::u128::MAX - 1)),
        ]
        .iter()
        .for_each(|(a, b, result)| {
            assert_eq!(widening_mul(*a, *b), *result);
        });
    }

    #[test]
    fn difficulty_check_works_for_valid_hashes() {
        [
            // Null hashes will always satisfy any difficulty. Including the maximum possible (u128::MAX)
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                std::u128::MAX,
            ),
            // The largest hash will satisfy the smallest difficulty
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                1,
            ),
            // Live difficulty and hash values from the Unprll mainnet
            (
                "7a03d4485600699035f5032f199dec212db1dca1ae386bfb141e1b52814f0000",
                126_000,
            ),
            (
                "b8ec4fec0e35df8656e2617f52f9c6a2e269cf14de3b1626fa3bcfa888550000",
                56_800,
            ),
            (
                "b3934917894808505d73785108461ccc3968600da02e1c2eaf26897f2bb60000",
                45_200,
            ),
            (
                "bd1e641c2eb5fb7aa47dcb484102e6578fe4c9d07bc983468082107b101b0100",
                25_600,
            ),
            (
                "66082ac23a926b7cb329f52a49cd60f0f69f419890db681f8d67eab01c510000",
                61_700,
            ),
        ]
        .iter()
        .map(|(hash, difficulty)| (Hash256::try_from(*hash).unwrap(), difficulty))
        .for_each(|(hash, difficulty)| {
            println!("Hash: {}, Difficulty: {}", hash, difficulty);
            assert!(check_hash_for_difficulty(hash.data(), *difficulty));
        });
    }

    #[test]
    fn difficulty_check_fails_for_invalid_hashes() {
        [
            // The largest hash will not satisfy any difficulty except 1
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                2,
            ),
            // Live difficulty and hash values from the Unprll mainnet
            (
                "97b18b0e547892c518f253f2f8f3debdfa50a1f5d727540032fcbdee57e324fd",
                126_000,
            ),
            (
                "cbc16aa0a9c9bc4f68902473a868f59ad4654d70173c79c764a6cf3f81ce6c4a",
                56_800,
            ),
            (
                "a50b0e393edb3ff51490b7745bb2fcba0185a195088a2bccf3e819da860a17be",
                45_200,
            ),
            (
                "23018812158f3a31066d1c464e60e5a6a64a3bbc7ee50aaee775061be7a379e3",
                25_600,
            ),
            (
                "ca0777d4b106820942dcab204cfcfd5d3e21671a2398a08772cb41fead395520",
                61_700,
            ),
        ]
        .iter()
        .map(|(hash, difficulty)| (Hash256::try_from(*hash).unwrap(), difficulty))
        .for_each(|(hash, difficulty)| {
            println!("Hash: {}, Difficulty: {}", hash, difficulty);
            assert!(!check_hash_for_difficulty(hash.data(), *difficulty));
        });
    }
}
