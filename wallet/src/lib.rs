use std::collections::HashMap;
use std::convert::From;

use byteorder::ByteOrder;
use serde::{
    Serialize,
    Deserialize
};

use common::{
    Transaction,
    TXExtra,
    TXOutTarget
};
use crypto::{
    CNFastHash,
    Digest,
    KeyPair,
    ScalarExt,
    SecretKey
};

pub mod address;

use address::Address;

#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SubAddressIndex(pub u32, pub u32);

#[derive(Serialize, Deserialize)]
pub struct Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    spend_keypair: KeyPair,
    view_keypair: KeyPair,

    addresses: HashMap<SubAddressIndex, Address<TCoinConfig>>
}

impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    /// Generate a wallet instance from a spend secret key and view secret key
    pub fn from_secret_keys(spend_secret_key: SecretKey, view_secret_key: SecretKey) -> Self {
        let mut w = Wallet {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
            addresses: HashMap::new()
        };

        // Insert standard address in the subaddresses map
        w.addresses.insert(SubAddressIndex(0, 0), Address::standard(w.spend_keypair.public_key, w.view_keypair.public_key));

        w
    }

    /// Deterministic wallet generation
    ///
    /// This allows having to store only one value (the spend secret key) while the others are computed
    /// The view secret key is derived by taking the Keccak (non-standard) hash of the spend secret key
    pub fn from(spend_secret_key: SecretKey) -> Self {
        let view_secret_key = SecretKey::from_slice(&CNFastHash::digest(&spend_secret_key.to_bytes()));

        Self::from_secret_keys(spend_secret_key, view_secret_key)
    }

    pub fn get_address_for_index(&self, index: &SubAddressIndex) -> Option<&Address<TCoinConfig>> {
        self.addresses.get(index)
    }

    pub fn spend_keypair(&self) -> &KeyPair {
        &self.spend_keypair
    }
    pub fn view_keypair(&self) -> &KeyPair {
        &self.view_keypair
    }
}

// Subaddresses
impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    pub fn add_new_subaddress(&mut self, index: SubAddressIndex) {
        // If we have an address already, don't regenerate it
        // (it's pointless for subaddresses, and it overwrites our standard address)
        if self.addresses.contains_key(&index) {
            return;
        }
        // Subaddress secret key
        let subaddress_secret_key = self.get_subaddress_secret_key(&index);
        let subaddress_public_key = subaddress_secret_key * crypto::ecc::BASEPOINT;

        // Subaddress spend public key
        let spend_public_key = self.spend_keypair.public_key.decompress().unwrap() + subaddress_public_key;

        // Subaddress view public key
        let view_public_key = self.view_keypair.secret_key * spend_public_key;

        // Compress public keys
        let spend_public_key = spend_public_key.compress();
        let view_public_key = view_public_key.compress();

        self.addresses.insert(index, Address::subaddress(spend_public_key, view_public_key));
    }

    fn get_subaddress_secret_key(&self, SubAddressIndex(major, minor): &SubAddressIndex) -> SecretKey {
        // m = H_s("SubAddr" | a | major | minor)
        // Length of buffer = length("SubAddr\0") + length(public_key) + 2 * length(u32)
        //                  = 8 + 32 + 8 = 48
        let mut buffer = [0; 48];

        // SubAddr
        buffer[..8].copy_from_slice(b"SubAddr\0");
        // View secret key
        buffer[8..40].copy_from_slice(self.view_keypair.secret_key.as_bytes());
        // Major index
        byteorder::LittleEndian::write_u32(&mut buffer[40..44], *major);
        // Minor index
        byteorder::LittleEndian::write_u32(&mut buffer[44..48], *minor);

        crypto::ecc::hash_to_scalar(CNFastHash::digest(&buffer))
    }
}

impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    pub fn scan_transaction(&mut self, transaction: &Transaction) -> Option<SecretKey> {
        for output in &transaction.prefix.outputs {
            let mut tx_pub_key = None;
            for extra in &transaction.prefix.extra {
                match extra {
                    TXExtra::TxPublicKey(key) => {
                        tx_pub_key = Some(key.decompress().unwrap());
                    }
                }
            };

            if let Some(tx_pub_key) = tx_pub_key {
                match output.target {
                    TXOutTarget::ToKey { key } => {
                        // Compute the common "tx scalar"
                        // H_s(aR)
                        let tx_scalar = crypto::ecc::data_to_scalar(&(self.view_keypair.secret_key * tx_pub_key));

                        // Do the original Cryptonote derivation first
                        // H_s(aR)G + B
                        let computed_pub_key = tx_scalar * crypto::ecc::BASEPOINT + self.spend_keypair.public_key.decompress().unwrap();

                        // Check if the output is to our standard address
                        let index_address_pair = if tx_pub_key == computed_pub_key {
                            // It's to our standard address
                            Some((&SubAddressIndex(0, 0), self.addresses.get(&SubAddressIndex(0, 0)).unwrap()))
                        } else {
                            // Try the subaddress derivation next
                            // P - H_s(aR)G
                            let computed_pub_key = key.decompress().unwrap() - tx_scalar * crypto::ecc::BASEPOINT;
                            let computed_pub_key = computed_pub_key.compress();

                            // Find the corresponding public spend key
                            self.addresses.iter().find(|(_, address)| {
                                address.spend_public_key == computed_pub_key
                            })
                        };

                        if let Some((index, _address)) = index_address_pair {
                            println!("Output found!");
                            let output_secret_key = if *index == SubAddressIndex(0, 0) {
                                // Main address derives things differently
                                // H_s(aR) + b
                                tx_scalar + self.spend_keypair.secret_key
                            } else {
                                // H_s(aR) + b + m_i
                                tx_scalar + self.spend_keypair.secret_key + self.get_subaddress_secret_key(index)
                            };

                            return Some(output_secret_key);
                        }
                    }
                }
            };
        }

        None
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
        let mut wallet: Wallet<TestPrefixes> = Wallet::from(SecretKey::from_slice(&hex::decode("67a2547fde618d6fbd4d450b28da58feb6836cf223c2f97980731448bb84c100").unwrap()));

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
            // Create the address at that index
            wallet.add_new_subaddress(index.clone());

            let address = wallet.get_address_for_index(&index).unwrap();
            (address.into(), address_str.to_string())
        }).for_each(|(computed_address, expected_address)| {
            // Should be equal to what it is on mainnet
            assert_eq!(
                computed_address,
                expected_address
            );
        });
    }

    #[test]
    fn it_receives_outputs_correctly() {
        // Test specific imports
        use common::{
            TransactionPrefix,
            TXExtra,
            TXOut,
            TXOutTarget
        };
        use address::AddressType;

        // A test wallet
        let mut wallet: Wallet<TestPrefixes> = Wallet::from(KeyPair::generate().secret_key);

        // TODO: Replace with actual transaction sending code
        [
            // The standard address recognizes inputs differently
            (0, 0),

            // Some subaddresses
            (1, 0),
            (1, 1),
            (100, 100),
            (32767, 256)
        ].iter()
        // Convert to SubAddressIndex
        .map(|(major, minor)| SubAddressIndex(*major, *minor))
        .for_each(|index| {
            // Add the subaddress and get its public keys
            wallet.add_new_subaddress(index.clone());
            let address = wallet.get_address_for_index(&index).unwrap();

            // r
            let random_scalar = KeyPair::generate().secret_key;

            let tx_pub_key = if let AddressType::Standard = address.address_type {
                // rG
                random_scalar * crypto::ecc::BASEPOINT
            } else {
                // rD
                random_scalar * address.spend_public_key.decompress().unwrap()
            };

            // H_s(rC)
            let tx_scalar = crypto::ecc::data_to_scalar(&(random_scalar * address.view_public_key.decompress().unwrap()));

            // H_s(rC)*G + D
            let tx_dest_key = tx_scalar * crypto::ecc::BASEPOINT + address.spend_public_key.decompress().unwrap();

            let t = Transaction {
                prefix: TransactionPrefix {
                    version: 1,
                    unlock_delta: 0,
                    inputs: Vec::default(),
                    outputs: vec!{
                        TXOut {
                            amount: 0,
                            target: TXOutTarget::ToKey {
                                key: tx_dest_key.compress()
                            }
                        }
                    },
                    extra: vec!{
                        TXExtra::TxPublicKey(tx_pub_key.compress())
                    }
                },
                signatures: Vec::new()
            };

            // Scan the transaction
            let tx_scan_result = wallet.scan_transaction(&t);

            // The tx output must be detected
            assert!(tx_scan_result.is_some());

            // The tx secret key must correspond to the tx destination key
            assert!(tx_scan_result.unwrap() * crypto::ecc::BASEPOINT == tx_dest_key);
        });
    }
}
