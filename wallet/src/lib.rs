use std::convert::From;

use byteorder::ByteOrder;

use crypto::{
    CNFastHash,
    Digest,
    KeyPair,
    ScalarExt,
    SecretKey
};

pub mod address;

use address::Address;

pub struct Wallet<TCoinConfig> {
    spend_keypair: KeyPair,
    view_keypair: KeyPair,
    marker: std::marker::PhantomData<TCoinConfig>
}

impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    /// Generate a wallet instance from a spend secret key and view secret key
    pub fn from_secret_keys(spend_secret_key: SecretKey, view_secret_key: SecretKey) -> Self {
        Wallet {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
            marker: std::marker::PhantomData
        }
    }

    /// Deterministic wallet generation
    ///
    /// This allows having to store only one value (the spend secret key) while the others are computed
    /// The view secret key is derived by taking the Keccak (non-standard) hash of the spend secret key
    pub fn from(spend_secret_key: SecretKey) -> Self {
        let view_secret_key = SecretKey::from_slice(&CNFastHash::digest(&spend_secret_key.to_bytes()));

        Self::from_secret_keys(spend_secret_key, view_secret_key)
    }

    pub fn get_address_for_index(&self, major: u32, minor: u32) -> Option<Address<TCoinConfig>> {
        if major == 0 && minor == 0 {
            Some(Address::standard(self.spend_keypair.public_key, self.view_keypair.public_key))
        } else {
            // Subaddress secret key
            let subaddress_secret_key = self.get_subaddress_secret_key(major, minor);
            let subaddress_public_key = subaddress_secret_key * crypto::ecc::BASEPOINT;

            // Subaddress spend public key
            let spend_public_key = self.spend_keypair.public_key.decompress().unwrap() + subaddress_public_key;

            // Subaddress view public key
            let view_public_key = self.view_keypair.secret_key * spend_public_key;

            // Compress public keys
            let spend_public_key = spend_public_key.compress();
            let view_public_key = view_public_key.compress();

            Some(Address::subaddress(spend_public_key, view_public_key))
        }
    }


    pub fn spend_keypair(&self) -> &KeyPair {
        &self.spend_keypair
    }
    pub fn view_keypair(&self) -> &KeyPair {
        &self.view_keypair
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct TestPrefixes;

    impl address::AddressPrefixConfig for TestPrefixes {
        const STANDARD:   u64 = 0x0014_5023; // UNP
        const SUBADDRESS: u64 = 0x0021_1023; // UNPS
        const INTEGRATED: u64 = 0x0029_1023; // UNPi
    }

    #[test]
    fn it_works() {
        let w: Wallet<TestPrefixes> = Wallet::from(SecretKey::from_slice(&hex::decode("91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a").unwrap()));

        // This given set of keys is that of a testnet wallet. As all keys are in public view,
        // DO NOT use this wallet for storing any coins

        // Spend private key (pedantic check)
        assert_eq!(
            hex::encode(w.spend_keypair().secret_key.to_bytes()),
            "91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a"
        );
        // Spend public key
        assert_eq!(
            hex::encode(w.spend_keypair().public_key.to_bytes()),
            "4dcff6ae0b5313938e718bb033907fee6cddc053f4d44c41bd0f9fed5ea7cef7"
        );

        // View secret key
        assert_eq!(
            hex::encode(w.view_keypair().secret_key.to_bytes()),
            "84bc8a0314bfa06dee4b992cca4420d19f28af37f4fb90e031454c66f8cd6003"
        );

        // View public key
        assert_eq!(
            hex::encode(w.view_keypair().public_key.to_bytes()),
            "8b66a0e272063786cc769c295486552e39797c57243612047bff9845c8cc66c8"
        );
    }

    #[test]
    fn it_generates_subaddress_keys_correctly() {
        // This given key is in public view, hence DO NOT use this wallet for storing any coins
        let wallet: Wallet<TestPrefixes> = Wallet::from(SecretKey::from_slice(&hex::decode("67a2547fde618d6fbd4d450b28da58feb6836cf223c2f97980731448bb84c100").unwrap()));

        [
            ((0, 1), "UNPStRsRsLKPPysVGYVe9fSHqxbAn4sN1RaRGVhGb4G5gpmt9JUzNhLaXndsFRUN3nGa6kzk7cViJBgAuB1dtBtjDKsTvY66vCL"),
            ((0, 2), "UNPStUCnafD3MwXfvYN2zCWfWFydyFyZxj89iLW481b8XcSdSV23Arz43ubi1UbBk6W2WNkCM3ysM1Ub2r8AQhAsCetDffLd6JK"),
            ((1, 0), "UNPStSrKaX54x6MPDmBtmTRE1bX7tZx3sYWGk877crypJ9KXT7qvcwpZDjtBioKwRz9CxBdZvZnob9CQ1K3QfvT6h1Jd81AdrjS"),
            ((1, 1), "UNPStUWbghuSyjDVJZvo3Y7MsYbk95JpVAUv9L72Wbh1HgVcqCgLxfhZaNHSwjcH42etkx1dnYYVb7jBXoER8J2ESHUbGQUTiWD"),
            ((2, 0), "UNPStRn7PHE6Qbx7QSThUeMzgKhuQXCN8VT9FUa2NqenBBgVfohskSLN739JU4tmHa5jUAgHD5JYYFh6wxNX2EbwPXeRwAa2XKR"),
            ((2, 1), "UNPStTzhL7Zc7Z7q4X5ZYxBEkpKmNJT6ojSAfcQ7jipq4HGvHMaQJPAg3BTt8PU4J16vvuPqnJzW28HfCuzJzpnHhbxKx7v9VKU")
        ].iter().map(|((major, minor), address_str)| -> (String, String) {
            (wallet.get_address_for_index(*major, *minor).unwrap().into(), address_str.to_string())
        }).for_each(|(computed_address, expected_address)| {
            // Should be equal to what it is on mainnet
            assert_eq!(
                computed_address,
                expected_address
            );
        });
    }

}
