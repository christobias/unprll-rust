use serde::{Serialize, Deserialize};

// Wallet Creation
#[derive(Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub file_name: String,
    pub password: String
}
