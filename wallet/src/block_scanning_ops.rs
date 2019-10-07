use common::{
    Block,
    Transaction,
    TXExtra,
    TXOutTarget
};
use crypto::{
    SecretKey
};

use crate::{
    address::AddressPrefixConfig,
    SubAddressIndex,
    Wallet
};

impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: AddressPrefixConfig
{
    pub fn scan_block(&mut self, block: &Block) {
        self.scan_transaction(&block.miner_tx);
    }

    fn scan_transaction(&mut self, transaction: &Transaction) -> Option<SecretKey> {
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
    use common::{
        TransactionPrefix,
        TXOut
    };
    use crypto::KeyPair;

    use crate::address::AddressType;
    use crate::test_definitions::TestCoin;

    use super::*;

    #[test]
    fn it_receives_outputs_correctly() {
        // A test wallet
        let mut wallet: Wallet<TestCoin> = Wallet::from(KeyPair::generate().secret_key);

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
