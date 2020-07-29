//! Module for scanning transactions

use crypto::{CNFastHash, Digest, KeyImage, KeyPair, PublicKey, SecretKey};

use crate::{
    account_keys::AccountKeys,
    derivation::Derivation,
    subaddress::{self, SubAddressIndex},
};

/// Gets the key image for the given transaction output
///
/// The key image is a tag used to prevent double spends of an output
pub fn get_key_image(
    account_keys: &AccountKeys,
    recipient_subaddress_index: &SubAddressIndex,
    output_destination: &PublicKey,
    output_tx_index: u64,
    output_tx_public_keys: &[PublicKey],
) -> Option<(KeyImage, KeyPair)> {
    // Get the output secret key. This will return None if the source output
    // doesn't belong to the account
    // x = H_s(arG || idx) + b
    let output_secret_key = get_output_secret_key(
        account_keys,
        recipient_subaddress_index,
        output_destination,
        output_tx_index,
        output_tx_public_keys,
    )?;

    // Generate the ephemeral keypair for this output (x, X = xG)
    let ephemeral_keypair = KeyPair::from(output_secret_key);

    // Generate the key image
    // KI = x * H_p(X)
    let key_image = ephemeral_keypair.secret_key
        * crypto::ecc::hash_to_point(CNFastHash::digest(
            ephemeral_keypair.public_key.compress().as_bytes(),
        ));

    // Check if the ephemeral keypair matches the output key
    if &ephemeral_keypair.public_key != output_destination {
        None
    } else {
        Some((key_image, ephemeral_keypair))
    }
}

/// Computes the output secret key needed for spending the given output
///
/// Returns the output secret key `H_s(arG || idx) + b` if it indeed is towards the account given
fn get_output_secret_key(
    account_keys: &AccountKeys,
    subaddress_index: &SubAddressIndex,
    output_destination: &PublicKey,
    tx_output_index: u64,
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
        let target_public_key =
            output_destination - (&derivation_scalar * &crypto::ecc::BASEPOINT_TABLE);

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
