//! API definitions for the RPC server

use serde::{Deserialize, Serialize};

/// Request the core's current status
#[derive(Serialize, Deserialize)]
pub struct GetStatsResponse {
    /// Current tail of the chain in the form of (height, block_id)
    pub tail: (u64, String),

    /// Target height for syncing
    pub target_height: u64,

    /// Current difficulty of the network
    pub difficulty: u128,

    /// Number of unconfirmed transactions in the mempool
    pub tx_pool_count: u64,
}

// get_blocks
/// Get the given block range from the main chain
#[derive(Serialize, Deserialize)]
pub struct GetBlocksRequest {
    /// Start height (inclusive)
    pub from: u64,
    /// End height (exclusive)
    pub to: Option<u64>,
}

// TODO: Make these strongly typed while still serializing to hex strings
/// Response to a GetBlocksRequest
#[derive(Serialize, Deserialize)]
pub struct GetBlocksResponse {
    /// Blocks in the form of hex strings
    pub blocks: Vec<String>,
    /// Transactions contained in the given blocks in the form of hex strings
    pub transactions: Vec<String>,
}
