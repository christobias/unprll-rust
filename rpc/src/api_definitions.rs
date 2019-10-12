use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GetStatsResponse {
    pub tail: (u64, String),
    pub target_height: u64,
    pub difficulty: u128,
    pub tx_pool_count: u64
}

// get_blocks

#[derive(Serialize, Deserialize)]
pub struct GetBlocksRequest {
    pub from: u64,
    pub to: Option<u64>
}

// TODO: Make these strongly typed while still serializing to hex strings
#[derive(Serialize, Deserialize)]
pub struct GetBlocksResponse {
    pub blocks: Vec<String>,
    pub transactions: Vec<String>
}
