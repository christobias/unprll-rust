use crate::hash::{self, Hash256};

fn tree_hash_cnt(count: usize) -> usize {
    assert!(count >= 3);
    assert!(count <= 0x10000000);

    let mut pow: usize = 2;
    while pow < count {
        pow <<= 1;
    }
    pow >> 1
}

pub fn tree_hash(hashes: &Vec<Hash256>) -> Hash256 {
    assert!(hashes.len() > 0);
    match hashes.len() {
        1 => hashes[0].clone(),
        2 => {
            let mut buf: [u8; 64] = [0; 64];
            buf[..32].copy_from_slice(&hashes[0].data());
            buf[32..].copy_from_slice(&hashes[1].data());
            hash::cn_fast_hash(&buf)
        },
        _ => {
            let mut cnt = tree_hash_cnt(hashes.len());
            let mut buf: Vec<u8> = Vec::with_capacity(cnt * 32);

            for i in 0..(2 * cnt - hashes.len()) {
                for val in hashes[i].data() {
                    buf.push(*val);
                }
            }

            for _i in (2 * cnt - hashes.len())..(cnt * 32) {
                buf.push(0);
            }

            let mut i: usize = 2 * cnt - hashes.len();
            for j in (2 * cnt - hashes.len())..cnt {
                let mut tmp: [u8; 64] = [0; 64];
                tmp[..32].copy_from_slice(&hashes[i    ].data());
                tmp[32..].copy_from_slice(&hashes[i + 1].data());
                let tmp = hash::cn_fast_hash(&tmp);
                &buf[(j * 32)..((j + 1) * 32)].copy_from_slice(tmp.data());
                i += 2;
            }
            assert!(i == hashes.len());

            while cnt > 2 {
                cnt >>= 1;
                let mut i = 0;
                for j in (0..(cnt * 32)).step_by(32) {
                    let tmp = hash::cn_fast_hash(&buf[i..(i + 64)]);
                    &buf[j..(j + 32)].copy_from_slice(tmp.data());
                    i += 64;
                }
            }

            hash::cn_fast_hash(&buf[..64])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let map: Vec<(Vec<Hash256>, Hash256)> = [
            (
                "676567f8b1b470207c20d8efbaacfa64b2753301b46139562111636f36304bb8",
                "676567f8b1b470207c20d8efbaacfa64b2753301b46139562111636f36304bb8"
            ),
            (
                "3124758667bc8e76e25403eee75a1044175d58fcd3b984e0745d0ab18f473984975ce54240407d80eedba2b395bcad5be99b5c920abc2423865e3066edd4847a",
                "5077570fed2363a14fa978218185b914059e23517faf366f08a87cf3c47fd58e"
            ),
            (
                "decc1e0aa505d7d5fbe8ed823d7f5da55307c4cc7008e306da82dbce492a0576dbcf0c26646d36b36a92408941f5f2539f7715bcb1e2b1309cedb86ae4211554f56f5e6b2fce16536e44c851d473d1f994793873996ba448dd59b3b4b922b183",
                "f8e26aaa7c36523cea4c5202f2df159c62bf70d10670c96aed516dbfd5cb5227"
            ),
            (
                "53edbbf98d3fa50a85fd2d46c42502aafad3fea30bc25ba4f16ec8bf4a475c4d87da8ad3e5c90aae0b10a559a77a0985608eaa3cc3dd338239be52572c3bdf4ba403d27466991997b3cf4e8d238d002a1451ccc9c4790269d0f0085d9382d60fef37717f59726e4cc8787d5d2d75238ba9adb9627a8f4aeeec8d80465ed3f5fb",
                "45f6e06fc0263e667caddd8fba84c9fb723a961a01a5b115f7cab7fe8f2c7e44"
            ),
            (
                "051a082e670c688e6a0fc2c8fd5b66b7a23cd380c7c49bd0cfffb0e80fb8c2334bb717c5e90db0ac353dfc0750c8b43a07edae0be99d6e820acc6da9f113123ae084c38ccdbf9c6730e228b5d98e7beb9843cfb523747cc32f09f2b16def67f76765cee044883827b9af31c179d3135b16c30f04453943d9676a59b907a6439658f6c98159b8fa1b152f1bcf748740754ca31c918501dbd577faf602c641df59",
                "e678fb87749ec082a9f92537716de8e19d8bd5bc4c4d832bd3fcfd42498dac83"
            ),
            (
                "4231b54cddc617d06e0e311536fa400e5be0a35aab5fec9ec8d98f6c6dad3916fe6cdb1f63be231f95cdc83bb15b0d99d32d9922331b738c423625471fad7f408e60c0773fe78938b054e28b86ac06a194d141c1bde5f3c6f2b11468b43702cb3121b40ccbcb5461fa9321c35c9342e21efd7c1c22f523d78b9d4de28112b6cc51552642ffc126c66f25038f9d3b0cf485cc252215c144d51a139c8ea9a0ecc16e81d8d92dd3660d885deca60070d3d00069d89db1a85acb9c1f18d0c90736a7",
                "7db3258ea536fef652eaaa9ccb158045770900b3c301d727bcb7e60f9831ae2c"
            ),
            (
                "68e09573a758b75ea8e7d925fe81e3155afecddc4c8aeb3fe70d87411ee53aceac63c0233d172cd49b2708350fd64e2cf4dccb13352e3a159c06647c609429349197163eca2c2dae0c8643fdfe5d346b2ffd45a2d46f38599efbfa587c3ac0c3119e19508e009556fe53e4f78ef30eed649cdc1e090c8cb662eae1863fdc683bbabea966764f550a142dd68e5b8eb1930ff0c7333c9f2555712489a8cf6a5d188a70841510fca540b8c0425123efca47d5a698cf392e3bdbb7226053459fae01fd19ddb9d16d5f5499525feb49ffca9411e7ac48de15256559f3f65f899b80af",
                "ad56b3e027d78a372adebe839e154668aec5236f7d40296cfdb562fca1dc73c2"
            ),
            (
                "42e7f4058ca80d513c140837dd661acde3fb914779079baccfe188cbce275aed4b515094bb49ab9a825bcc2ac13f84b14a9defeb1b62fc68124b088272a3562696d62ccdfb5d896b2d2b410a2a79f9b1e7849feebc17617ba12a08d4e80affe970ff2fb79917ac13708f79be215bb6484d298b2fe22b4818536e74894db5e0350e1505ca2681da7b7d7171e3d10c89348cab160ff5b2e739d3591443d2af60db5eb36c50a2dfdb79b8ab83b0792161ac4756d9b831f1863188e10c81af5077d0fdb123f66e51670f03a203ff2287dea6827dcd5afd4904736ec4fe9f3b52f7e2bed7beaa1543bd8bfbfff6a8ae8bf1791dc34efa92c6342532fa33a3b72b6c9f",
                "090a95612ed9df6eeb854ae320355889a302498b4f5164a79d8e384a3a0d9748"
            )
        ].iter().map(|x| {
            let buf = hex::decode(x.0).unwrap();
            let mut vec = Vec::default();
            for i in (0..buf.len()).step_by(32) {
                let mut hash = Hash256::null_hash();
                hash.copy_from_slice(&buf[i..(i + 32)]);
                vec.push(hash);
            }
            (vec, Hash256::from(x.1).unwrap())
        }).collect();

        let child = std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                for (data, hash) in map.iter() {
                    assert_eq!(tree_hash(data), *hash);
                }
            }).unwrap();
        child.join().unwrap();
    }
}
