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
use crate::api_definitions::Stats;

type CoreRef = Arc<RwLock<CryptonoteCore>>;

pub fn build_server(core: CoreRef) -> Result<Server<CoreRef>, Error> {
    let s = Server::with_state(core)
        .with_method("get_stats", get_stats)
        .with_method("submit_block", submit_block)
        .finish();

    Ok(s)
}

fn get_stats(state: State<CoreRef>) -> Result<Stats, Error> {
    let state = state.read().unwrap();
    let blockchain = state.blockchain();

    Ok(Stats {
        difficulty: 100,
        tail: blockchain.get_tail().map(|x| (x.0, x.1.get_hash().to_string())).unwrap(),
        net_type: "mainnet".to_owned(),
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
