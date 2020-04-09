use std::sync::{Arc, RwLock};

use jsonrpsee::{
    common::Error,
    raw::RawServer,
    transport::TransportServer,
};

use crate::api_definitions::*;
use common::GetHash;
use cryptonote_core::{CryptonoteCore, EmissionCurve};

type CoreRef<TCoin> = Arc<RwLock<CryptonoteCore<TCoin>>>;

pub struct DaemonRPCServer<R, I, TCoin>
where
    R: TransportServer<RequestId = I>,
    I: Clone + Eq + std::hash::Hash + Send + Sync,
    TCoin: EmissionCurve,
{
    core: CoreRef<TCoin>,
    server: Arc<RwLock<RawServer<R, I>>>,
}

impl<R, I, TCoin> DaemonRPCServer<R, I, TCoin>
where
    R: TransportServer<RequestId = I>,
    I: Clone + Eq + std::hash::Hash + Send + Sync,
    TCoin: EmissionCurve,
{
    pub fn new(server: RawServer<R, I>, core: CoreRef<TCoin>) -> Self {
        Self {
            core,
            server: Arc::from(RwLock::from(server)),
        }
    }
    pub async fn run(self) {
        while let Ok(request) = DaemonRPC::next_request(&mut self.server.write().unwrap()).await {
            match request {
                DaemonRPC::GetStats { respond } => {
                    let response = async {
                        let core = self.core.read().unwrap();
                        let blockchain = core.blockchain();

                        Ok::<_, failure::Error>(GetStatsResponse {
                            difficulty: 1,
                            tail: blockchain
                                .get_tail()
                                .map(|x| (x.0, x.1.get_hash().to_string()))?,
                            target_height: 9999,
                            tx_pool_count: 0,
                        })
                    };
                    match response.await {
                        Ok(response) => respond.ok(response).await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }

                DaemonRPC::SubmitBlock { respond, block } => {
                    let response = async {
                        let block = bincode::deserialize(&hex::decode(block)?)?;

                        let mut core = self.core.write().unwrap();
                        let blockchain = core.blockchain_mut();
    
                        blockchain.add_new_block(block)?;
    
                        Ok::<_, failure::Error>(())
                    };
                    match response.await {
                        Ok(()) => respond.ok("").await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }

                DaemonRPC::GetBlocks { respond, from: start_height, to: end_height } => {
                    let response = async {
                        // The end height is optional and will default to a specified value. If the request is too
                        // large, the range is reduced
                        // TODO: Implement range reduction

                        let core = self.core.read().unwrap();
                        let blockchain = core.blockchain();

                        let end_height = end_height
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

                        Ok::<_, failure::Error>(GetBlocksResponse {
                            blocks: blocks
                                .into_iter()
                                .flat_map(|block| bincode_epee::serialize(&block))
                                .map(hex::encode)
                                .collect(),
                            transactions,
                        })
                    };
                    match response.await {
                        Ok(response) => respond.ok(response).await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }
            }
        }
    }
}
