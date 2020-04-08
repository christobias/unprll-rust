use std::sync::{Arc, RwLock};

use jsonrpc_v2::{Data, Error, MapRouter, Params, Server};

use crate::api_definitions::*;
use common::GetHash;
use cryptonote_core::{CryptonoteCore, EmissionCurve};

type CoreRef<TCoin> = Arc<RwLock<CryptonoteCore<TCoin>>>;

pub fn build_server<TCoin: 'static + EmissionCurve + Send + Sync>(
    core: CoreRef<TCoin>,
) -> Result<Arc<Server<MapRouter>>, Error> {
    let s = Server::new()
        .with_data(Data::new(core))
        .with_method("get_stats", get_stats::<TCoin>)
        .with_method("submit_block", submit_block::<TCoin>)
        .with_method("get_blocks", get_blocks::<TCoin>)
        .finish();

    Ok(s)
}

async fn get_stats<TCoin>(data: Data<CoreRef<TCoin>>) -> Result<GetStatsResponse, Error>
where
    TCoin: EmissionCurve,
{
    let data = data.read().unwrap();
    let blockchain = data.blockchain();

    Ok(GetStatsResponse {
        difficulty: 10,
        tail: blockchain
            .get_tail()
            .map(|x| (x.0, x.1.get_hash().to_string()))
            .unwrap(),
        target_height: 9999,
        tx_pool_count: 0,
    })
}

async fn submit_block<TCoin>(
    Params(params): Params<Vec<String>>,
    data: Data<CoreRef<TCoin>>,
) -> Result<(), Error>
where
    TCoin: EmissionCurve,
{
    let block = bincode::deserialize(&hex::decode(&params[0])?)?;

    let mut data = data.write().unwrap();
    let blockchain = data.blockchain_mut();

    blockchain.add_new_block(block)?;

    Ok(())
}

async fn get_blocks<TCoin>(
    Params(params): Params<GetBlocksRequest>,
    data: Data<CoreRef<TCoin>>,
) -> Result<GetBlocksResponse, Error>
where
    TCoin: EmissionCurve,
{
    let start_height = params.from;
    // The end height is optional and will default to a specified value. If the request is too
    // large, the range is reduced
    // TODO: Implement range reduction

    let data = data.read().unwrap();
    let blockchain = data.blockchain();

    let end_height = params
        .to
        .unwrap_or_else(|| blockchain.get_tail().unwrap().0);

    let blocks = blockchain.get_blocks(start_height, end_height);

    let transactions = blocks
        .iter()
        .flat_map(|block| &block.tx_hashes)
        .map(|txid| {
            hex::encode(
                bincode_epee::serialize(&blockchain.get_transaction(txid).expect(
                    "The blockchain must always have all transactions from confirmed blocks",
                ))
                .unwrap(),
            )
        })
        .collect();

    Ok(GetBlocksResponse {
        blocks: blocks
            .into_iter()
            .flat_map(|block| bincode_epee::serialize(&block))
            .map(hex::encode)
            .collect(),
        transactions,
    })
}
