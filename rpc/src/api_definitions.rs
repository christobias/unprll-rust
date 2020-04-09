// Needed because jsonrpsee generates unused variables
#![allow(unused_variables)]
// Needed because jsonrpsee doesn't allow documenting members just yet
#![allow(missing_docs)]

//! API definitions for the RPC server

use serde::{Deserialize, Serialize};

jsonrpsee::rpc_api! {
    pub DaemonRPC {
        /// Request the core's current status
        fn get_stats() -> GetStatsResponse;

        /// Submit a mined block to the chain
        fn submit_block(block: String) -> String;

        /// Request a range of confirmed blocks from the blockchain
        fn get_blocks(from: u64, to: Option<u64>) -> GetBlocksResponse;
    }
}

/// Core's current status
#[derive(Serialize, Deserialize)]
pub struct GetStatsResponse {
    /// Current tail of the chain in the form of (height, block_id)
    pub tail: (u64, String),

    /// Target height for syncing
    pub target_height: u64,

    /// Current difficulty of the network
    // TODO FIXME: Change back to u128 once jsonrpsee works with it
    pub difficulty: u64,

    /// Number of unconfirmed transactions in the mempool
    pub tx_pool_count: u64,
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
