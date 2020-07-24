// Needed because jsonrpsee generates unused variables
#![allow(unused_variables)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

jsonrpsee::rpc_api! {
    pub WalletRPC {
        // Wallet management
        fn create_wallet(wallet_name: String, password: String) -> String;
        fn load_wallet(wallet_name: String, password: String) -> String;
        fn refresh_wallets() -> String;
        fn save_wallets() -> String;

        // Account and Address management
        fn get_addresses(wallet_name: String, account_index: u32, address_indices: Option<Vec<u32>>) -> GetAddressesResponse;
        fn get_balances(wallet_name: String, account_indices: Vec<u32>) -> GetBalancesResponse;
    }
}

// [get_addresses]
#[derive(Default, Serialize, Deserialize)]
pub struct GetAddressesResponse {
    pub addresses: HashMap<u32, String>,
}

// [get_balances]
#[derive(Default, Serialize, Deserialize)]
pub struct GetBalancesResponse {
    pub balances: HashMap<u32, u64>,
}
