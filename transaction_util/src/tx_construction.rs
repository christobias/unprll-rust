//! Module for constructing Cryptonote transactions

use rand::{seq::SliceRandom, RngCore};

use common::{GetHash, TXExtra, TXIn, TXNonce, TXOut, TXOutTarget, Transaction, TransactionPrefix};
use crypto::{ecc::Scalar, CNFastHash, Digest, Hash8, Hash8Data, KeyImage, KeyPair, PublicKey};
use ensure_macro::ensure;
use ringct::{Error as RingCTError, RingCTInput, RingCTOutput};

use crate::{
    account_keys::AccountKeys, address::AddressType, derivation::Derivation, payment_id,
    subaddress, tx_scanning, TXDestination, TXDestinationType, TXSource,
};

/// Error type for transaction construction
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returned when there are no transaction sources
    #[error("No transaction sources")]
    NoSources,

    /// Returned when there are no transaction destinations
    #[error("No transaction destinations")]
    NoDestinations,

    /// Returned when the index of the real output is beyond the mixin set provided
    #[error("Real output index is beyond output mixin set")]
    RealIndexOutOfBounds,

    /// Returned when the key image could not be generated
    #[error("Key image could not be generated for given output")]
    KeyImageGeneration,

    /// Returned when the transaction has more than one payment ID
    #[error("Transaction has more than one payment ID")]
    MultiplePaymentIDs,

    /// Returned when the sum of output amounts is greater than the sum of input amounts
    #[error("Transaction spends more than it contains as input")]
    ExcessSpending,

    /// Returned when the payment ID could not be encrypted
    #[error("Payment ID could not be encrypted")]
    PaymentIDEncryption,

    /// Returned when there is an error when creating the RingCT signature
    #[error(transparent)]
    RingCT(#[from] RingCTError)
}

type Result<T> = std::result::Result<T, Error>;

/// Generates a key image for the given input to the transaction
///
/// The key image is a tag used to prevent double spends of a transaction input
fn generate_key_image(
    account_keys: &AccountKeys,
    source: &TXSource,
) -> Option<(KeyImage, KeyPair)> {
    let (_, real_output) = source.outputs[source.real_output_index as usize];

    // Get the output secret key. This will return None if the source output
    // doesn't belong to the account
    // x = H_s(arG || idx) + b
    let output_secret_key = tx_scanning::get_output_secret_key(
        account_keys,
        &source.subaddress_index,
        source.real_output_tx_index,
        real_output.destination,
        &source.real_output_tx_public_keys,
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
    if ephemeral_keypair.public_key != real_output.destination {
        None
    } else {
        Some((key_image, ephemeral_keypair))
    }
}

/// Finds the public key for the main destination of a transaction
///
/// Used for encrypting payment IDs
fn find_destination_public_key(destinations: &[TXDestination]) -> Option<PublicKey> {
    let mut dest_pub_key = None;

    for dest in destinations {
        // Check for a non-empty, non-change destination
        if dest.amount != 0 {
            if let TXDestinationType::PayToAddress(address) = &dest.destination_type {
                if let Some(current_key) = dest_pub_key {
                    // Check if the last found address is not the same as this one
                    if current_key != address.view_public_key {
                        return None;
                    }
                }
                // Keep track of it
                dest_pub_key = Some(address.view_public_key);
            }
        }
    }

    dest_pub_key
}

/// Constructs a transaction spending the given sources towards the given destinations
pub fn construct_tx(
    sender_keys: &AccountKeys,
    sources: &mut [TXSource],
    destinations: &mut [TXDestination],
    unlock_delta: u16,
) -> Result<(Transaction, Vec<Scalar>)> {
    // Perform sanity checks

    // Ensure we have inputs
    ensure!(!sources.is_empty(), Error::NoSources);

    // Ensure we have outputs
    ensure!(!destinations.is_empty(), Error::NoDestinations);

    // Handle the inputs
    let mut in_amount_sum = 0;
    let mut tx_inputs = Vec::new();
    for source in sources {
        // Check if all real inputs are within range
        ensure!(
            (source.real_output_index as usize) < source.outputs.len(),
            Error::RealIndexOutOfBounds
        );

        // Add the current amount
        in_amount_sum += source.amount;

        // Generate key image
        // x * H_p(P)
        let (key_image, ephemeral_keypair) = generate_key_image(&sender_keys, source)
            .ok_or_else(|| Error::KeyImageGeneration)?;

        // Convert absolute offsets to relative
        let key_offsets = source.outputs.iter().fold(Vec::new(), |mut acc, (pos, _)| {
            acc.push(pos - acc.last().unwrap_or(&0));
            acc
        });

        // Store the new input
        tx_inputs.push((
            source,
            TXIn::FromKey {
                key_image,
                key_offsets,
            },
            ephemeral_keypair,
        ));
    }

    // Sort the inputs (and corresponding sources and ephemeral keys) by key image
    tx_inputs.sort_unstable_by(|(_, tx_in_a, _), (_, tx_in_b, _)| {
        if let (
            TXIn::FromKey {
                key_image: a_ki, ..
            },
            TXIn::FromKey {
                key_image: b_ki, ..
            },
        ) = (tx_in_a, tx_in_b)
        {
            a_ki.compress().as_bytes().cmp(b_ki.compress().as_bytes())
        } else {
            unreachable!("We have no other possible situations here!")
        }
    });

    // Shuffle the outputs to avoid identification of outputs
    destinations.shuffle(&mut rand::rngs::OsRng);

    // Classify our destinations
    //
    // Also grab the payment ID from Integrated addresses
    let mut num_standard = 0;
    let mut num_subaddr = 0;
    let mut payment_id = None;
    for dest in destinations.iter() {
        // Ignore change addresses. They pay to the sender
        if let TXDestinationType::PayToAddress(address) = &dest.destination_type {
            match &address.address_type {
                AddressType::Standard => num_standard += 1,
                AddressType::SubAddress => num_subaddr += 1,
                AddressType::Integrated(current_payment_id) => {
                    // Check if we've got a payment ID already
                    ensure!(payment_id.is_none(), Error::MultiplePaymentIDs);

                    payment_id = Some(current_payment_id.clone());
                    // Integrated addresses are standard addresses
                    num_standard += 1;
                }
            }
        }
    }

    // Generate a payment ID if we're not given any
    let payment_id = payment_id.unwrap_or_else(|| {
        let mut hash = [0; 8];
        rand::rngs::OsRng.fill_bytes(&mut hash);
        Hash8::from(Hash8Data::clone_from_slice(&hash))
    });

    // Create the transaction keypair (r, R = rG)
    //
    // If the payment is towards one or more standard addresses or a single subaddress
    // this is the only keypair needed
    let mut tx_keypair = KeyPair::generate();

    if num_standard == 0 && num_subaddr == 1 {
        // If we have a single subaddress destination, set the transaction public key to sD
        let dest_addr = destinations.iter().fold(None, |mut dest, curr| {
            if let TXDestinationType::PayToAddress(address) = &curr.destination_type {
                if let AddressType::SubAddress = address.address_type {
                    // This is probably unnecessary but could act as a failsafe
                    if dest.is_some() {
                        unreachable!("We should never have the single subaddress destination case when multiple subaddresses are present");
                    }
                    dest = Some(address);
                }
            }
            dest
        }).unwrap();

        tx_keypair.public_key = tx_keypair.secret_key * dest_addr.spend_public_key;
    }
    // If not, then we use the default tx public key as generated with the keypair (R = rG)

    // Generate additional transaction keypairs if needed
    //
    // More than one key is needed when there are multiple subaddress destinations (ignoring change)
    // or a subaddress along with a standard address
    let need_additional_tx_keypairs = num_subaddr > 0 && (num_standard > 0 || num_subaddr > 1);
    let mut additional_tx_keypairs = Vec::new();

    let mut out_amount_sum = 0;
    let mut amount_secret_keys = Vec::new();
    let mut tx_outputs = Vec::new();
    for (output_index, dest) in destinations.iter().enumerate() {
        // Regardless of destination type, generate an additional tx keypair if needed
        let destination_address = match &dest.destination_type {
            TXDestinationType::Change(subaddress_index) => {
                subaddress::get_address_for_index(&sender_keys, &subaddress_index)
            }
            TXDestinationType::PayToAddress(address) => address.clone(),
        };

        let additional_tx_keypair = if need_additional_tx_keypairs {
            // Each additional keypair is generated as before, as (r, R = rG)
            let mut kp = KeyPair::generate();
            if let AddressType::SubAddress = destination_address.address_type {
                // Change the public key to sD
                kp.public_key = kp.secret_key * destination_address.spend_public_key;
            }

            Some(kp)
        } else {
            None
        };

        // Store the generated transaction keypair
        if need_additional_tx_keypairs {
            additional_tx_keypairs.push(additional_tx_keypair.clone());
        }

        let derivation = match &dest.destination_type {
            TXDestinationType::Change(_) => {
                // Change to ourselves, aR = arG
                Derivation::from(&sender_keys.view_keypair.secret_key, &tx_keypair.public_key)
            }
            TXDestinationType::PayToAddress(address) => {
                // Paying a different address, rA
                let secret_key = if address.address_type == AddressType::SubAddress
                    && need_additional_tx_keypairs
                {
                    additional_tx_keypair.unwrap().secret_key
                } else {
                    tx_keypair.secret_key
                };

                Derivation::from(&secret_key, &address.view_public_key)
            }
        }
        .unwrap();

        // Generate the target keypair
        // (H_s(rA || idx), H_s(rA || idx)G + B)
        // This is derivable by both receiver and sender
        let target_keypair =
            derivation.to_keypair(output_index as u64, destination_address.spend_public_key);

        // Store the amount secret key used for encrypting amounts in RingCT
        amount_secret_keys.push(target_keypair.secret_key);

        // Add this amount to the cumulative sum
        out_amount_sum += dest.amount;

        // Store this output
        tx_outputs.push(TXOut {
            amount: dest.amount,
            target: TXOutTarget::ToKey {
                key: target_keypair.public_key,
            },
        });
    }

    // Check if the transaction is spending more than its inputs
    ensure!(out_amount_sum <= in_amount_sum, Error::ExcessSpending);

    // Find the output key of the target recipient
    // H_s(arG || idx)G + B
    let destination_public_key = find_destination_public_key(destinations).unwrap();

    // Keep a copy of the transaction secret keys for future use
    let mut transaction_secret_keys = vec![tx_keypair.secret_key];
    if need_additional_tx_keypairs {
        transaction_secret_keys.extend(
            additional_tx_keypairs
                .iter()
                .map(|kp| kp.as_ref().unwrap().secret_key),
        )
    }

    let mut tx_extra = vec![
        // Encrypt the payment ID with the single output tx secret key
        //
        // The derivation from below is derivable by both sender and receiver
        // since, rA = aR = arG
        // TODO: Consider multi-payment ID scenarios
        TXExtra::TxNonce(TXNonce::EncryptedPaymentId(payment_id::encrypt(
            payment_id,
            Derivation::from(&tx_keypair.secret_key, &destination_public_key)
                .ok_or_else(|| Error::PaymentIDEncryption)?,
        ))),
        // Store the transaction public key
        TXExtra::TxPublicKey(tx_keypair.public_key),
    ];
    if need_additional_tx_keypairs {
        // Store the additional transaction public keys
        tx_extra.push(TXExtra::TxAdditionalPublicKeys(
            additional_tx_keypairs
                .into_iter()
                .map(|x| x.unwrap().public_key)
                .collect::<Vec<_>>(),
        ))
    }

    // Convert the transaction inputs into a form recognizable by RingCT
    let rct_inputs = tx_inputs
        .iter()
        .map(|(source, _, ephemeral_keypair)| RingCTInput {
            destination_secret_key: ephemeral_keypair.secret_key,
            commitment_secret_key: source.amount_mask,
            amount: source.amount,
            ring_index: source.real_output_index,
            ring_row: source.outputs.iter().map(|(_, pair)| *pair).collect(),
        })
        .collect::<Vec<_>>();

    // Similarly for outputs
    let rct_outputs = tx_outputs
        .iter()
        .zip(amount_secret_keys.iter())
        .map(|(out, &amount_secret_key)| {
            let destination_public_key = match out.target {
                TXOutTarget::ToKey { key } => key,
            };

            RingCTOutput {
                destination_public_key,
                amount: out.amount,
                amount_secret_key,
            }
        })
        .collect::<Vec<_>>();

    // Create the transaction prefix
    let tx_prefix = TransactionPrefix {
        version: 2,
        unlock_delta,
        extra: tx_extra,
        inputs: tx_inputs.into_iter().map(|(_, input, _)| input).collect(),
        outputs: tx_outputs,
    };

    // Compute the RingCT signature (simple variant)
    let ringct_signature = ringct::sign(
        tx_prefix.get_hash(),
        &rct_inputs,
        &rct_outputs,
        in_amount_sum - out_amount_sum,
    )?;

    Ok((
        Transaction {
            prefix: tx_prefix,
            rct_signature: Some(ringct_signature),
        },
        transaction_secret_keys,
    ))
}

#[cfg(test)]
pub mod tests {
    use ringct::{Commitment, DestinationCommitmentPair};

    use crate::SubAddressIndex;

    use super::*;

    /// Creates a TXSource targeted to the sender
    pub fn create_mock_source(
        tx_keypair: &KeyPair,
        sender_keys: &AccountKeys,
        sender_subaddress_index: SubAddressIndex,
        amount: u64,
        mixin_ring_size: u64,
    ) -> TXSource {
        // Get the sender's address
        let sender_address =
            subaddress::get_address_for_index(sender_keys, &sender_subaddress_index);

        // Generate random indices
        let real_output_index = rand::rngs::OsRng.next_u64() % mixin_ring_size;
        let real_output_tx_index = rand::rngs::OsRng.next_u64();

        // Create a derivation that points to the sender
        let derivation =
            Derivation::from(&tx_keypair.secret_key, &sender_address.view_public_key).unwrap();

        // Generate the target keypair
        let target_keypair =
            derivation.to_keypair(real_output_tx_index, sender_address.spend_public_key);

        // Create the commitment to the amount
        let commitment = Commitment::commit_to_value(amount);

        let outputs = (0..mixin_ring_size)
            .map(|i| {
                (
                    (i + 1) * mixin_ring_size,
                    if i == real_output_index {
                        DestinationCommitmentPair {
                            destination: target_keypair.public_key,
                            commitment: commitment.clone().into_public(),
                        }
                    } else {
                        DestinationCommitmentPair {
                            destination: KeyPair::generate().public_key,
                            commitment: KeyPair::generate().public_key,
                        }
                    },
                )
            })
            .collect();

        let source = TXSource {
            amount,
            amount_mask: commitment.mask,
            outputs,
            real_output_index,
            real_output_tx_index,
            real_output_tx_public_keys: vec![tx_keypair.public_key],
            subaddress_index: SubAddressIndex(0, 0),
        };

        // Make sure this output actually does go to the sender
        assert!(tx_scanning::get_output_secret_key(
            &sender_keys,
            &SubAddressIndex(0, 0),
            real_output_tx_index,
            source.outputs[real_output_index as usize].1.destination,
            &source.real_output_tx_public_keys,
        )
        .is_some());

        source
    }

    #[test]
    fn it_creates_transactions_correctly() {
        // Create the sender's keys
        let sender_keys = AccountKeys::from(KeyPair::generate().secret_key);

        // Create a transaction keypair
        let tx_keypair = KeyPair::generate();

        // Create 2 sources
        let mut sources = (1..=2)
            .map(|i| create_mock_source(&tx_keypair, &sender_keys, SubAddressIndex(0, 0), i, 10))
            .collect::<Vec<_>>();

        // Create 2 destinations. One to an address, another to change
        let mut destinations = vec![
            TXDestination {
                amount: 1,
                destination_type: TXDestinationType::PayToAddress(
                    subaddress::get_address_for_index(&sender_keys, &SubAddressIndex(1, 0)),
                ),
            },
            TXDestination {
                amount: 1,
                destination_type: TXDestinationType::Change(SubAddressIndex(1, 0)),
            },
        ];

        let (tx, _) = construct_tx(&sender_keys, &mut sources, &mut destinations, 4).unwrap();

        let rct_signature = tx.rct_signature.unwrap();
        // Check for the right fee
        assert_eq!(rct_signature.base.fee, 1);

        // Ensure the RingCT signature is correct
        ringct::verify_multiple(&[rct_signature]).unwrap();

        // Check if both outputs from the created transaction do indeed reach the sender
        for (output_index, output) in tx.prefix.outputs.iter().enumerate() {
            let tx_public_keys = tx
                .prefix
                .extra
                .iter()
                .filter_map(move |extra| match extra {
                    TXExtra::TxPublicKey(key) => Some(vec![*key]),
                    TXExtra::TxAdditionalPublicKeys(keys) => Some(keys.to_vec()),
                    TXExtra::TxNonce(_) => None,
                })
                .flat_map(|x| x.into_iter())
                .collect::<Vec<_>>();

            let TXOutTarget::ToKey { key: output_key } = output.target;

            assert!(tx_scanning::get_output_secret_key(
                &sender_keys,
                &SubAddressIndex(1, 0),
                output_index as u64,
                output_key,
                &tx_public_keys,
            )
            .is_some());
        }
    }

    #[test]
    fn it_rejects_overspending() {
        // Create the sender's keys
        let sender_keys = AccountKeys::from(KeyPair::generate().secret_key);

        // Create a transaction keypair
        let tx_keypair = KeyPair::generate();

        // Create 2 sources adding up to 3 input coins
        let mut sources = (1..=2)
            .map(|i| create_mock_source(&tx_keypair, &sender_keys, SubAddressIndex(0, 0), i, 10))
            .collect::<Vec<_>>();

        // Create a single destination spending 4 coins
        let mut destinations = vec![TXDestination {
            amount: 4,
            destination_type: TXDestinationType::PayToAddress(subaddress::get_address_for_index(
                &sender_keys,
                &SubAddressIndex(1, 0),
            )),
        }];

        assert!(construct_tx(&sender_keys, &mut sources, &mut destinations, 4).is_err());
    }
}
