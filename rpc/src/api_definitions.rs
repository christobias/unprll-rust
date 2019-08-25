use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub tail: (u64, String),
    pub target_height: u64,
    pub difficulty: u64,
    pub net_type: String,
    pub tx_pool_count: u64
}
