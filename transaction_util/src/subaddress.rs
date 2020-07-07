//! Module for handling subaddresses

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};

use crate::{AccountKeys, Address};
use crypto::{CNFastHash, Digest, SecretKey};

/// Tuple of (major, minor) index for a subaddress
#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SubAddressIndex(pub u32, pub u32);

/// Get the address at a given index from the current wallet
pub fn get_address_for_index(account_keys: &AccountKeys, index: &SubAddressIndex) -> Address {
    if index == &SubAddressIndex(0, 0) {
        return Address::standard(
            account_keys.spend_keypair.public_key,
            account_keys.view_keypair.public_key,
        );
    }
    // Subaddress secret key
    let subaddress_secret_key = get_subaddress_secret_key(account_keys, &index);
    let subaddress_public_key = &subaddress_secret_key * &crypto::ecc::BASEPOINT_TABLE;

    // Subaddress spend public key
    let spend_public_key =
        account_keys.spend_keypair.public_key.decompress().unwrap() + subaddress_public_key;

    // Subaddress view public key
    let view_public_key = account_keys.view_keypair.secret_key * spend_public_key;

    // Compress public keys
    let spend_public_key = spend_public_key.compress();
    let view_public_key = view_public_key.compress();

    Address::subaddress(spend_public_key, view_public_key)
}

/// Get the secret key used in generating a subaddress in the given index
pub fn get_subaddress_secret_key(
    account_keys: &AccountKeys,
    SubAddressIndex(major, minor): &SubAddressIndex,
) -> SecretKey {
    // m = H_s("SubAddr" | a | major | minor)
    // Length of buffer = length("SubAddr\0") + length(public_key) + 2 * length(u32)
    //                  = 8 + 32 + 8 = 48
    let mut buffer = [0; 48];

    // SubAddr
    buffer[..8].copy_from_slice(b"SubAddr\0");
    // View secret key
    buffer[8..40].copy_from_slice(account_keys.view_keypair.secret_key.as_bytes());
    // Major index
    byteorder::LittleEndian::write_u32(&mut buffer[40..44], *major);
    // Minor index
    byteorder::LittleEndian::write_u32(&mut buffer[44..48], *minor);

    crypto::ecc::hash_to_scalar(CNFastHash::digest(&buffer))
}

#[cfg(test)]
mod tests {
    use crypto::ScalarExt;

    use super::*;
    use crate::test_definitions::TestCoin;

    #[test]
    fn it_generates_subaddress_keys_correctly() {
        // This given key is in public view, hence DO NOT use this wallet for storing any coins
        let account_keys: AccountKeys = AccountKeys::from(SecretKey::from_slice(
            &hex::decode("67a2547fde618d6fbd4d450b28da58feb6836cf223c2f97980731448bb84c100")
                .unwrap(),
        ));

        [
            ((0, 1), "UNPStRsRsLKPPysVGYVe9fSHqxbAn4sN1RaRGVhGb4G5gpmt9JUzNhLaXndsFRUN3nGa6kzk7cViJBgAuB1dtBtjDKsTvY66vCL"),
            ((0, 2), "UNPStUCnafD3MwXfvYN2zCWfWFydyFyZxj89iLW481b8XcSdSV23Arz43ubi1UbBk6W2WNkCM3ysM1Ub2r8AQhAsCetDffLd6JK"),
            ((1, 0), "UNPStSrKaX54x6MPDmBtmTRE1bX7tZx3sYWGk877crypJ9KXT7qvcwpZDjtBioKwRz9CxBdZvZnob9CQ1K3QfvT6h1Jd81AdrjS"),
            ((1, 1), "UNPStUWbghuSyjDVJZvo3Y7MsYbk95JpVAUv9L72Wbh1HgVcqCgLxfhZaNHSwjcH42etkx1dnYYVb7jBXoER8J2ESHUbGQUTiWD"),
            ((2, 0), "UNPStRn7PHE6Qbx7QSThUeMzgKhuQXCN8VT9FUa2NqenBBgVfohskSLN739JU4tmHa5jUAgHD5JYYFh6wxNX2EbwPXeRwAa2XKR"),
            ((2, 1), "UNPStTzhL7Zc7Z7q4X5ZYxBEkpKmNJT6ojSAfcQ7jipq4HGvHMaQJPAg3BTt8PU4J16vvuPqnJzW28HfCuzJzpnHhbxKx7v9VKU")
        ].iter().map(|((major, minor), address_str)| {
            (SubAddressIndex(*major, *minor), address_str)
        }).map(|(index, address_str)| -> (String, _) {
            // Get the address at that index
            let address = get_address_for_index(&account_keys, &index);
            (address.to_address_string::<TestCoin>(), (*address_str).to_string())
        }).for_each(|(computed_address, expected_address)| {
            // Should be equal to what it is on mainnet
            assert_eq!(
                computed_address,
                expected_address
            );
        });
    }
}
