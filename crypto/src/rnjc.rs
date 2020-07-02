use byteorder::ByteOrder;

use blake_hash::Blake256;
use groestl_aesni::Groestl256;
use jh_x86_64::Jh256;
use sha3::{Digest, Keccak256Full};
use skein_hash::Skein512;

use crate::cast_256::{self, Cast256Key};
use crate::hash::Hash256Data;

const MEMORY: usize = 1 << 20;
const ITER: u32 = 1024;
const RECURSION_DEPTH: u32 = 2;
const RECURSION_ITER: u32 = 4;

const CAST256_BLOCK_SIZE: usize = 16;
const INIT_SIZE_BLK: usize = 8;
const INIT_SIZE_BYTE: usize = INIT_SIZE_BLK * CAST256_BLOCK_SIZE;

type SlowHashState = [u8; 200];

#[inline(always)]
fn xor_blocks(a: &mut [u8], b: &[u8]) {
    let mut a0: u128 = byteorder::LittleEndian::read_u128(a);
    let b0: u128 = byteorder::LittleEndian::read_u128(b);
    a0 ^= b0;
    byteorder::LittleEndian::write_u128(a, a0);
}

#[inline(always)]
fn swap_blocks(a: &mut [u8], b: &mut [u8]) {
    // Very helpfully, blocks are 128 bits wide, and all we need is a u128
    let mut a0: u128 = byteorder::LittleEndian::read_u128(a);
    let mut b0: u128 = byteorder::LittleEndian::read_u128(b);
    std::mem::swap(&mut a0, &mut b0);
    byteorder::LittleEndian::write_u128(a, a0);
    byteorder::LittleEndian::write_u128(b, b0);
}

#[inline(always)]
fn e2i(a: &[u8], count: usize) -> usize {
    (byteorder::LittleEndian::read_u64(a) as usize / CAST256_BLOCK_SIZE) & (count - 1)
}

#[inline(always)]
fn apply_hash(data: &[u8], n: u8) -> Hash256Data {
    match n {
        0 => Blake256::digest(data),
        1 => Groestl256::digest(data),
        2 => Jh256::digest(data),
        3 => Skein512::digest(data),
        _ => panic!("This shouldn't happen!"),
    }
}

/// The RNJC hash function
pub struct RNJC {
    data_buffer: Vec<u8>,
}

impl RNJC {
    fn rnjc_recursive(data: &[u8], recursion_depth: u32) -> Hash256Data {
        // Scratchpad
        let mut long_state: [u8; MEMORY] = [0; MEMORY];
        // Hash state
        let mut hash_state: SlowHashState = [0; 200];
        // CAST256 key
        let mut cast256_key: Cast256Key = [0; 8];
        // "Register" blocks
        let mut reg_a: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
        let mut reg_b: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
        let mut reg_c: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];
        let mut reg_d: [u8; CAST256_BLOCK_SIZE] = [0; CAST256_BLOCK_SIZE];

        // Fill hash state
        hash_state[..200].copy_from_slice(&Keccak256Full::digest(data));

        // Fill initializer buffer
        let mut text: [u32; INIT_SIZE_BYTE / 4] = [0; INIT_SIZE_BYTE / 4];
        byteorder::LittleEndian::read_u32_into(
            &hash_state[64..(64 + INIT_SIZE_BYTE)],
            &mut text[..],
        );
        // Fill key
        byteorder::LittleEndian::read_u32_into(&hash_state[..32], &mut cast256_key[..]);
        let mut cast256_key = cast_256::get_key_schedule(cast256_key);

        // Fill scratchpad
        for i in 0..(MEMORY / INIT_SIZE_BYTE) {
            for j in 0..INIT_SIZE_BLK {
                let res = &cast_256::encrypt(
                    &text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))],
                    &cast256_key,
                );
                text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))]
                    .copy_from_slice(res);
            }
            byteorder::LittleEndian::write_u32_into(
                &text,
                &mut long_state[(INIT_SIZE_BYTE * i)..(INIT_SIZE_BYTE * (i + 1))],
            );
        }

        // Initialize register blocks
        for i in 0..16 {
            reg_a[i] = hash_state[i] ^ hash_state[32 + i];
            reg_b[i] = hash_state[16 + i] ^ hash_state[48 + i];
        }

        for i in 0..ITER {
            let index: usize = e2i(&reg_a[..8], MEMORY / CAST256_BLOCK_SIZE);
            reg_c.copy_from_slice(
                &long_state[(CAST256_BLOCK_SIZE * index)..(CAST256_BLOCK_SIZE * (index + 1))],
            );
            match (u32::from(reg_a[0]) ^ (i * recursion_depth)) & 3 {
                0 => {
                    // CAST256 Encrypt
                    let mut buf: Cast256Key = [0; 8];
                    byteorder::LittleEndian::read_u32_into(&reg_a[..], &mut buf[..4]);
                    cast256_key = cast_256::get_key_schedule(buf);
                    byteorder::LittleEndian::read_u32_into(&reg_c[..], &mut buf[..4]);
                    byteorder::LittleEndian::write_u32_into(
                        &cast_256::encrypt(&buf[..4], &cast256_key),
                        &mut reg_c[..],
                    );
                }
                1 => {
                    // Multiply
                    let a1: u128 = byteorder::LittleEndian::read_u64(&reg_a[..8]).into();
                    let c1: u128 = byteorder::LittleEndian::read_u64(&reg_c[..8]).into();
                    let res: u128 = a1 * c1;
                    byteorder::LittleEndian::write_u128(&mut reg_d[..], res);
                    let d1: u64 = byteorder::LittleEndian::read_u64(&reg_d[..8]);
                    let d2: u64 = byteorder::LittleEndian::read_u64(&reg_d[8..]);
                    byteorder::LittleEndian::write_u64(&mut reg_d[..8], d2);
                    byteorder::LittleEndian::write_u64(&mut reg_d[8..], d1);
                    // Half-add
                    let mut a0: u64 = byteorder::LittleEndian::read_u64(&reg_b[..8]);
                    let mut a1: u64 = byteorder::LittleEndian::read_u64(&reg_b[8..]);
                    let b0: u64 = byteorder::LittleEndian::read_u64(&reg_d[..8]);
                    let b1: u64 = byteorder::LittleEndian::read_u64(&reg_d[8..]);
                    a0 = a0.wrapping_add(b0);
                    a1 = a1.wrapping_add(b1);
                    byteorder::LittleEndian::write_u64(&mut reg_b[..8], a0);
                    byteorder::LittleEndian::write_u64(&mut reg_b[8..], a1);
                }
                2 => {
                    let tmp = apply_hash(&reg_c, reg_a[0] & 3);
                    reg_c.copy_from_slice(&tmp[..16]);
                }
                3 => {
                    // CAST256 Decrypt
                    let mut buf: Cast256Key = [0; 8];
                    byteorder::LittleEndian::read_u32_into(&reg_a[..], &mut buf[..4]);
                    cast256_key = cast_256::get_key_schedule(buf);
                    byteorder::LittleEndian::read_u32_into(&reg_c[..], &mut buf[..4]);
                    byteorder::LittleEndian::write_u32_into(
                        &cast_256::decrypt(&buf[..4], &cast256_key),
                        &mut reg_c[..],
                    );
                }
                _ => unreachable!(),
            }
            xor_blocks(&mut reg_b, &reg_c);
            swap_blocks(&mut reg_b, &mut reg_c);
            long_state[(CAST256_BLOCK_SIZE * index)..(CAST256_BLOCK_SIZE * (index + 1))]
                .copy_from_slice(&reg_c);
            assert_eq!(index, e2i(&reg_a[..8], MEMORY / CAST256_BLOCK_SIZE));
            swap_blocks(&mut reg_a, &mut reg_b);
        }

        // Recursion
        if recursion_depth > 0 {
            for i in 0..RECURSION_ITER {
                // Iteration 1
                let j = e2i(&reg_a[..8], MEMORY / CAST256_BLOCK_SIZE);
                reg_c.copy_from_slice(
                    &long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))],
                );
                let tmp_hash = RNJC::rnjc_recursive(&reg_a, recursion_depth - 1);
                if i % 2 == 0 {
                    reg_c.copy_from_slice(&tmp_hash[..CAST256_BLOCK_SIZE]);
                } else {
                    reg_c.copy_from_slice(&tmp_hash[CAST256_BLOCK_SIZE..]);
                }
                xor_blocks(&mut reg_b, &reg_c);
                swap_blocks(&mut reg_b, &mut reg_c);
                long_state[(CAST256_BLOCK_SIZE * j)..(CAST256_BLOCK_SIZE * (j + 1))]
                    .copy_from_slice(&reg_c);
                assert_eq!(j, e2i(&reg_a[..8], MEMORY / CAST256_BLOCK_SIZE));
                swap_blocks(&mut reg_a, &mut reg_b);
            }
        }

        // Fill initializer buffer
        let mut text: [u32; INIT_SIZE_BYTE / 4] = [0; INIT_SIZE_BYTE / 4];
        byteorder::LittleEndian::read_u32_into(
            &hash_state[64..(64 + INIT_SIZE_BYTE)],
            &mut text[..],
        );
        // Fill key
        let mut cast256_key: Cast256Key = [0; 8];
        byteorder::LittleEndian::read_u32_into(&hash_state[32..64], &mut cast256_key[..]);
        let cast256_key = cast_256::get_key_schedule(cast256_key);

        // Fill scratchpad
        for i in 0..(MEMORY / INIT_SIZE_BYTE) {
            for j in 0..INIT_SIZE_BLK {
                byteorder::LittleEndian::write_u32_into(
                    &text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))],
                    &mut reg_b,
                );
                xor_blocks(
                    &mut reg_b,
                    &long_state[(i * INIT_SIZE_BYTE + j * CAST256_BLOCK_SIZE)
                        ..(i * INIT_SIZE_BYTE + j * CAST256_BLOCK_SIZE + CAST256_BLOCK_SIZE)],
                );
                byteorder::LittleEndian::read_u32_into(
                    &reg_b,
                    &mut text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))],
                );

                let res = &cast_256::encrypt(
                    &text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))],
                    &cast256_key,
                );
                text[((CAST256_BLOCK_SIZE / 4) * j)..((CAST256_BLOCK_SIZE / 4) * (j + 1))]
                    .copy_from_slice(res);
            }
        }
        byteorder::LittleEndian::write_u32_into(
            &text[..],
            &mut hash_state[64..(64 + INIT_SIZE_BYTE)],
        );

        let mut tmp: [u64; 25] = [0; 25];
        byteorder::LittleEndian::read_u64_into(&hash_state, &mut tmp);
        keccak::f1600(&mut tmp);
        byteorder::LittleEndian::write_u64_into(&tmp, &mut hash_state);

        apply_hash(&hash_state, hash_state[0] & 3)
    }

    #[inline(always)]
    fn rnjc(data: &[u8]) -> Hash256Data {
        RNJC::rnjc_recursive(data, RECURSION_DEPTH)
    }
}

impl Digest for RNJC {
    type OutputSize = digest::generic_array::typenum::U32;
    fn new() -> Self {
        RNJC {
            data_buffer: Vec::default(),
        }
    }
    fn input<B: AsRef<[u8]>>(&mut self, data: B) {
        self.data_buffer.extend_from_slice(data.as_ref());
    }
    fn chain<B: AsRef<[u8]>>(self, _data: B) -> Self {
        unimplemented!()
    }
    fn result(self) -> Hash256Data {
        RNJC::rnjc(&self.data_buffer)
    }
    fn result_reset(&mut self) -> Hash256Data {
        let h = RNJC::rnjc(&self.data_buffer);
        self.reset();
        h
    }
    fn reset(&mut self) {
        self.data_buffer.clear();
        self.data_buffer.shrink_to_fit();
    }
    fn digest(data: &[u8]) -> Hash256Data {
        RNJC::rnjc(data)
    }
    fn output_size() -> usize {
        32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let map: std::collections::HashMap<Vec<u8>, Vec<u8>> = [
            (
                "6465206f6d6e69627573206475626974616e64756d",
                "e3885ed5133600d18fae678619908d004a9e9d4b939bf16d3acefcd03c40b601",
            ),
            (
                "6162756e64616e732063617574656c61206e6f6e206e6f636574",
                "f547066b684f510aa65416c30ea6353c9a61f9554826a440a3f3e47ee6ddeb4d",
            ),
            (
                "63617665617420656d70746f72",
                "cf60103f7fbf8da22b04b2780206cd77f34deab373a7e4a39b111670b9ba428a",
            ),
            (
                "6578206e6968696c6f206e6968696c20666974",
                "30651d2bc3651887ba7e252ec79188addd5c12758b667d18616b743e64751fc4",
            ),
        ]
        .iter()
        .map(|(data, hash)| (hex::decode(data).unwrap(), hex::decode(hash).unwrap()))
        .collect();

        let child = std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                for (data, hash) in map.iter() {
                    assert_eq!(RNJC::digest(data)[..], hash[..]);
                }
            })
            .unwrap();
        child.join().unwrap();
    }
}
