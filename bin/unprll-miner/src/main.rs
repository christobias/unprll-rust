use std::convert::TryFrom;
use futures::future::Future;

mod miner;
mod network;

use miner::Miner;
use network::Network;

fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .build()
        .unwrap();

    let mut m = Miner::new();

    let f = futures::future::lazy(move || {
        Network::new().unwrap().client.do_rpc("get_stats", &[])
    }).map_err(|x| eprintln!("{}", x))
    .and_then(|stats: rpc::api_definitions::Stats| {
        let mut b = common::Block::genesis();
        b.miner_tx.prefix.inputs[0] = common::TXIn::Gen { height: stats.tail.0 + 1 };
        b.header.prev_id = crypto::Hash256::try_from(stats.tail.1.as_str()).unwrap();
        m.set_block(b);
        m
    })
    .map(move |block| {
        Network::new().unwrap().client.do_rpc("submit_block", &[jsonrpc::serde_json::Value::String(hex::encode(bincode::serialize(&block).unwrap()))]).unwrap()
    }).map(|result: String| {
        println!("{}", result);
    });

    runtime.spawn(f.map_err(|_| {}));

    runtime.shutdown_on_idle().wait().unwrap();
}
