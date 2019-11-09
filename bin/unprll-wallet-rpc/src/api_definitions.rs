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

// [get_addresses]
#[derive(Default, Serialize, Deserialize)]
pub struct GetAddressesRequest {
    pub wallet_name: String,
    pub account_index: u32,
    pub address_indices: Option<Vec<u32>>
}

#[derive(Default, Serialize, Deserialize)]
pub struct GetAddressesResponse {
    pub addresses: HashMap<u32, String>
}
