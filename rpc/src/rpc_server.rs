use std::sync::{
    Arc,
    RwLock
};

use jsonrpc_v2::{
    Error,
    Params,
    Server,
    State
};

use common::GetHash;
use cryptonote_core::CryptonoteCore;
use crate::api_definitions::*;

type CoreRef = Arc<RwLock<CryptonoteCore>>;

pub fn build_server(core: CoreRef) -> Result<Server<CoreRef>, Error> {
    let s = Server::with_state(core)
        .with_method("get_stats", get_stats)
        .with_method("submit_block", submit_block)
        .with_method("get_blocks", get_blocks)
        .finish();

    Ok(s)
}

fn get_stats(state: State<CoreRef>) -> Result<GetStatsResponse, Error> {
    let state = state.read().unwrap();
    let blockchain = state.blockchain();

    Ok(GetStatsResponse {
        difficulty: 1,
        tail: blockchain.get_tail().map(|x| (x.0, x.1.get_hash().to_string())).unwrap(),
        target_height: 9999,
        tx_pool_count: 0
    })
}

fn submit_block(Params(params): Params<Vec<String>>, state: State<CoreRef>) -> Result<(), Error> {
    let block = bincode::deserialize(&hex::decode(&params[0])?)?;

    let mut state = state.write().unwrap();
    let blockchain = state.blockchain_mut();

    blockchain.add_new_block(block)?;

    Ok(())
}

fn get_blocks(Params(params): Params<GetBlocksRequest>, state: State<CoreRef>) -> Result<GetBlocksResponse, Error> {
    let start_height = params.from;
    // The end height is optional and will default to a specified value. If the request is too
    // large, the range is reduced
    // TODO: Implement range reduction

    let state = state.read().unwrap();
    let blockchain = state.blockchain();

    let end_height = params.to.unwrap_or_else(|| blockchain.get_tail().unwrap().0);

    let blocks = blockchain.get_blocks(start_height, end_height)
        .ok_or(jsonrpc_v2::Error::INVALID_PARAMS)?;

    let transactions = blocks
        .iter()
        .flat_map(|block| &block.tx_hashes).map(|txid| {
            hex::encode(
                bincode_epee::serialize(
                    &blockchain.get_transaction(txid)
                        .expect("The blockchain must always have all transactions from confirmed blocks")
                ).unwrap()
            )
        }).collect();

    Ok(GetBlocksResponse {
        blocks: blocks
            .into_iter()
            .flat_map(|block| bincode_epee::serialize(&block))
            .map(hex::encode)
            .collect(),
        transactions
    })
}
