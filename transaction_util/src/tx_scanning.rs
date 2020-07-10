//! Module for scanning transactions

use crypto::{PublicKey, SecretKey};

use crate::{
    account_keys::AccountKeys,
    derivation::Derivation,
    subaddress::{self, SubAddressIndex},
};

/// Computes the output secret key needed for spending the given output
///
/// Returns the output secret key `H_s(arG || idx) + b` if it indeed is towards the account given
pub fn get_output_secret_key(
    account_keys: &AccountKeys,
    subaddress_index: &SubAddressIndex,
    tx_output_index: u64,
    output_key: PublicKey,
    tx_public_keys: &[PublicKey],
) -> Option<SecretKey> {
    let key_derivations = tx_public_keys.iter().map(|tx_public_key| {
        // aR = rA = arG
        Derivation::from(&account_keys.view_keypair.secret_key, tx_public_key)
    });

    for derivation in key_derivations {
        let derivation = derivation?;

        // H_s(aR || idx) + b
        let derivation_scalar = derivation.to_scalar(tx_output_index);
        let address = subaddress::get_address_for_index(account_keys, subaddress_index);

        // If the output is indeed towards us
        // H_s(rA || idx)G + B - dG
        // = H_s(rA || idx)G + B - H_s(aR || idx)G
        // = (H_s(arG || idx) - H_s(arG || idx))G + B
        // = B
        let target_public_key = output_key - (&derivation_scalar * &crypto::ecc::BASEPOINT_TABLE);

        if target_public_key == address.spend_public_key {
            // H_s(arG || idx) + b
            let mut output_secret_key =
                derivation.to_scalar(tx_output_index) + account_keys.spend_keypair.secret_key;

            if subaddress_index != &SubAddressIndex(0, 0) {
                // Subaddresses require an extra addition for the subaddress secret key
                // H_s(aR) + b + m_i
                output_secret_key +=
                    subaddress::get_subaddress_secret_key(&account_keys, &subaddress_index)
            };

            return Some(output_secret_key);
        }
    }

    None
}
