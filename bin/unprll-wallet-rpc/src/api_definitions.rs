use std::collections::HashMap;

use serde::{Serialize, Deserialize};

// [create_wallet]
#[derive(Default, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub wallet_name: String,
    pub password: String
}

// [load_wallet]
#[derive(Default, Serialize, Deserialize)]
pub struct LoadWalletRequest {
    pub wallet_name: String,
    pub password: String
}

// [get_address]
#[derive(Default, Serialize, Deserialize)]
pub struct GetAddressRequest {
    pub wallet_name: String,
    pub account_index: u64,
    pub address_indices: Option<Vec<u64>>
}

#[derive(Default, Serialize, Deserialize)]
pub struct GetAddressResponse {
    pub addresses: HashMap<u64, String>
}
