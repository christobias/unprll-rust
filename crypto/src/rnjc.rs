use byteorder::ByteOrder;

use blake_hash::Blake256;
use jh_x86_64::Jh256;
use groestl_aesni::Groestl256;
use sha3::{Digest, Keccak256Full};
use keccak;
use skein_hash::Skein512;

use crate::hash::Hash256;
use crate::cast_256;

const MEMORY: usize = 1 << 20;
const ITER: u32 = 1024;
const RECURSION_DEPTH: u32 = 2;
const RECURSION_ITER: u32 = 4;

const CAST256_BLOCK_SIZE: usize = 16;
const INIT_SIZE_BLK: usize = 8;
const INIT_SIZE_BYTE: usize = INIT_SIZE_BLK * CAST256_BLOCK_SIZE;

type SlowHashState = [u8; 200];

fn xor_blocks(a: &mut [u8], b: &[u8]) {
    for i in 0..CAST256_BLOCK_SIZE {
        a[i] ^= b[i];
    }
}

fn swap_blocks(a: &mut [u8], b: &mut [u8]) {
    // Very helpfully, blocks are 128 bits wide, and all we need is a u128
    let mut a0: u128 = byteorder::LittleEndian::read_u128(a);
    let mut b0: u128 = byteorder::LittleEndian::read_u128(b);
    std::mem::swap(&mut a0, &mut b0);
    byteorder::LittleEndian::write_u128(a, a0);
    byteorder::LittleEndian::write_u128(b, b0);
}

fn e2i(a: &[u8], count: usize) -> usize {
    (byteorder::LittleEndian::read_u64(a) as usize / CAST256_BLOCK_SIZE) & (count - 1)
}

fn apply_hash(data: &[u8], n: u8) -> Hash256 {
    let mut hash = Hash256::null_hash();
    hash.copy_from_slice(&(match n {
        0 => Blake256::digest(data),
        1 => Groestl256::digest(data),
        2 => Jh256::digest(data),
        3 => Skein512::digest(data),
        _ => panic!("This shouldn't happen!")
    }));
    hash
}

fn rnjc_recursive(data: &[u8], recursion_depth: u32) -> Hash256 {
    // Scratchpad
    let mut long_state: [u8; MEMORY] = [0; MEMORY];
    // Hash state
    let mut hash_state: SlowHashState = [0; 200];
    // CAST256 key
    let mut cast256_key: cast_256::Cast256Key = [0; 8];
    // "Register" blocks
    let mut a: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
    let mut b: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
    let mut c: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
    let mut d: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];

    // Fill hash state
    hash_state[..200].copy_from_slice(&Keccak256Full::digest(data));

    // Fill initializer buffer
    let mut text: [u32; (INIT_SIZE_BYTE / 4)] = [0; (INIT_SIZE_BYTE / 4)];
    byteorder::LittleEndian::read_u32_into(&hash_state[64..(64 + INIT_SIZE_BYTE)], &mut text[..]);
    // Fill key
    byteorder::LittleEndian::read_u32_into(&hash_state[..32], &mut cast256_key[..]);
    let mut cast256_key = cast_256::get_key_schedule(cast256_key);

    // Fill scratchpad
    for i in 0..(MEMORY / INIT_SIZE_BYTE) {
        for j in 0..INIT_SIZE_BLK {
            let res = &cast_256::encrypt(&text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))], cast256_key);
            text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))].copy_from_slice(res);
        }
        byteorder::LittleEndian::write_u32_into(&text, &mut long_state[(INIT_SIZE_BYTE * i)..(INIT_SIZE_BYTE * (i + 1))]);
    }

    // Initialize register blocks
    for i in 0..16 {
        a[i] = hash_state[     i] ^ hash_state[32 + i];
        b[i] = hash_state[16 + i] ^ hash_state[48 + i];
    }

    for i in 0..ITER {
        let j: usize = e2i(&a[..8], MEMORY / CAST256_BLOCK_SIZE);
        c.copy_from_slice(&long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))]);
        let n = (a[0] as u32 ^ (i * recursion_depth)) & 3;
        match n {
            0 => {
                // CAST256 Encrypt
                let mut buf: cast_256::Cast256Key = [0; 8];
                byteorder::LittleEndian::read_u32_into(&a[..], &mut buf[..4]);
                cast256_key = cast_256::get_key_schedule(buf);
                byteorder::LittleEndian::read_u32_into(&c[..], &mut buf[..4]);
                byteorder::LittleEndian::write_u32_into(&cast_256::encrypt(&buf[..4], cast256_key), &mut c[..]);
            },
            1 => {
                // Multiply
                let a1: u128 = byteorder::LittleEndian::read_u64(&a[..8]).into();
                let c1: u128 = byteorder::LittleEndian::read_u64(&c[..8]).into();
                let res: u128 = a1 * c1;
                byteorder::LittleEndian::write_u128(&mut d[..], res);
                let d1: u64 = byteorder::LittleEndian::read_u64(&d[..8]);
                let d2: u64 = byteorder::LittleEndian::read_u64(&d[8..]);
                byteorder::LittleEndian::write_u64(&mut d[..8], d2);
                byteorder::LittleEndian::write_u64(&mut d[8..], d1);
                // Half-add
                let mut a0: u64 = byteorder::LittleEndian::read_u64(&b[..8]);
                let mut a1: u64 = byteorder::LittleEndian::read_u64(&b[8..]);
                let b0: u64 = byteorder::LittleEndian::read_u64(&d[..8]);
                let b1: u64 = byteorder::LittleEndian::read_u64(&d[8..]);
                a0 = a0.wrapping_add(b0);
                a1 = a1.wrapping_add(b1);
                byteorder::LittleEndian::write_u64(&mut b[..8], a0);
                byteorder::LittleEndian::write_u64(&mut b[8..], a1);
            },
            2 => {
                let tmp = apply_hash(&c, a[0] & 3);
                c.copy_from_slice(&tmp.data()[..16]);
            },
            3 => {
                // CAST256 Decrypt
                let mut buf: cast_256::Cast256Key = [0; 8];
                byteorder::LittleEndian::read_u32_into(&a[..], &mut buf[..4]);
                cast256_key = cast_256::get_key_schedule(buf);
                byteorder::LittleEndian::read_u32_into(&c[..], &mut buf[..4]);
                byteorder::LittleEndian::write_u32_into(&cast_256::decrypt(&mut buf[..4], cast256_key), &mut c[..]);
            },
            _ => panic!("This must never happen!")
        }
        xor_blocks(&mut b, &mut c);
        swap_blocks(&mut b, &mut c);
        long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))].copy_from_slice(&c);
        assert_eq!(j, e2i(&a[..8], MEMORY / CAST256_BLOCK_SIZE));
        swap_blocks(&mut a, &mut b);
    }

    // Recursion
    if recursion_depth > 0 {
        for i in 0..RECURSION_ITER {
            // Iteration 1
            let j = e2i(&a[..8], MEMORY / CAST256_BLOCK_SIZE);
            c.copy_from_slice(&long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))]);
            let tmp_hash: Hash256 = rnjc_recursive(&a, recursion_depth - 1);
            if i % 2 == 0 {
                c.copy_from_slice(&tmp_hash.data()[..CAST256_BLOCK_SIZE]);
            } else {
                c.copy_from_slice(&tmp_hash.data()[CAST256_BLOCK_SIZE..]);
            }
            xor_blocks(&mut b, &mut c);
            swap_blocks(&mut b, &mut c);
            long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))].copy_from_slice(&c);
            assert_eq!(j, e2i(&a[..8], MEMORY / CAST256_BLOCK_SIZE));
            swap_blocks(&mut a, &mut b);
        }
    }

    // Fill initializer buffer
    let mut text: [u32; (INIT_SIZE_BYTE / 4)] = [0; (INIT_SIZE_BYTE / 4)];
    byteorder::LittleEndian::read_u32_into(&hash_state[64..(64 + INIT_SIZE_BYTE)], &mut text[..]);
    // Fill key
    let mut cast256_key: cast_256::Cast256Key = [0; 8];
    byteorder::LittleEndian::read_u32_into(&hash_state[32..64], &mut cast256_key[..]);
    let cast256_key = cast_256::get_key_schedule(cast256_key);

    // Fill scratchpad
    for i in 0..(MEMORY / INIT_SIZE_BYTE) {
        for j in 0..INIT_SIZE_BLK {
            byteorder::LittleEndian::write_u32_into(&text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))], &mut b);
            xor_blocks(&mut b, &mut long_state[(i * INIT_SIZE_BYTE + j * CAST256_BLOCK_SIZE)..(i * INIT_SIZE_BYTE + j * CAST256_BLOCK_SIZE + CAST256_BLOCK_SIZE)]);
            byteorder::LittleEndian::read_u32_into(&b, &mut text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))]);

            let res = &cast_256::encrypt(&text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))], cast256_key);
            text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))].copy_from_slice(res);
        }
    }
    byteorder::LittleEndian::write_u32_into(&text[..], &mut hash_state[64..(64 + INIT_SIZE_BYTE)]);

    let mut tmp: [u64; 25] = [0; 25];
    byteorder::LittleEndian::read_u64_into(&hash_state, &mut tmp);
    keccak::f1600(&mut tmp);
    byteorder::LittleEndian::write_u64_into(&tmp, &mut hash_state);

    let n: u8 = hash_state[0] & 3;
    apply_hash(&hash_state, n)
}

pub fn rnjc(data: &[u8]) -> Hash256 {
    rnjc_recursive(data, RECURSION_DEPTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let map: std::collections::HashMap<Vec<u8>, Hash256> = [
            ("6465206f6d6e69627573206475626974616e64756d",            "e3885ed5133600d18fae678619908d004a9e9d4b939bf16d3acefcd03c40b601"),
            ("6162756e64616e732063617574656c61206e6f6e206e6f636574",  "f547066b684f510aa65416c30ea6353c9a61f9554826a440a3f3e47ee6ddeb4d"),
            ("63617665617420656d70746f72",                            "cf60103f7fbf8da22b04b2780206cd77f34deab373a7e4a39b111670b9ba428a"),
            ("6578206e6968696c6f206e6968696c20666974",                "30651d2bc3651887ba7e252ec79188addd5c12758b667d18616b743e64751fc4")
        ].iter().map(|x| {
            (hex::decode(x.0).unwrap(), Hash256::from(x.1).unwrap())
        }).collect();

        let child = std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                for (data, hash) in map.iter() {
                    assert_eq!(rnjc(data), *hash);
                }
            }).unwrap();
        child.join().unwrap();
    }
}
