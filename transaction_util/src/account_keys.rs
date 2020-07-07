use serde::{Deserialize, Serialize};

use crypto::{CNFastHash, Digest, KeyPair, ScalarExt, SecretKey};

#[derive(Deserialize, Serialize)]
/// A combination of a view and spend keypair which is used to create and recognize transactions
pub struct AccountKeys {
    /// Spend keypair
    pub spend_keypair: KeyPair,
    /// View keypair
    pub view_keypair: KeyPair,
}

/// Deterministic keypair generation
///
/// The view secret key is derived by taking the Keccak (non-standard) hash of the spend secret key
impl From<SecretKey> for AccountKeys {
    fn from(spend_secret_key: SecretKey) -> AccountKeys {
        let view_secret_key =
            SecretKey::from_slice(&CNFastHash::digest(spend_secret_key.as_bytes()));

        AccountKeys {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
        }
    }
}

impl AccountKeys {
    /// Generate an account keypair with distinct view and secret keys
    pub fn from_non_deterministic_keys(
        spend_secret_key: SecretKey,
        view_secret_key: SecretKey,
    ) -> AccountKeys {
        AccountKeys {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
        }
    }
}
