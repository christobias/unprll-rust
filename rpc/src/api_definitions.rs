use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GetStatsResponse {
    pub tail: (u64, String),
    pub target_height: u64,
    pub difficulty: u128,
    pub tx_pool_count: u64
}
