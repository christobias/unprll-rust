use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use futures::{Async, Poll, Stream};
use libp2p::PeerId;

use common::GetHash;
use cryptonote_core::{CryptonoteCore, EmissionCurve};

use super::protocol::{CryptonoteP2PMessage, NodeInfo};

pub struct CryptonoteP2PHandler<TCoin>
where
    TCoin: EmissionCurve,
{
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
    peers: HashMap<PeerId, Option<NodeInfo>>,
    pending_messages: VecDeque<(PeerId, CryptonoteP2PMessage)>,
}

impl<TCoin> CryptonoteP2PHandler<TCoin>
where
    TCoin: EmissionCurve,
{
    pub fn new(core: Arc<RwLock<CryptonoteCore<TCoin>>>) -> Self {
        Self {
            core,
            peers: HashMap::new(),
            pending_messages: VecDeque::new(),
        }
    }

    fn send_event(&mut self, peer_id: PeerId, message: CryptonoteP2PMessage) {
        self.pending_messages.push_back((peer_id, message));
    }

    pub fn add_new_peer(&mut self, peer_id: PeerId) {
        self.send_event(peer_id.clone(), CryptonoteP2PMessage::GetInfo);
        self.peers.insert(peer_id.clone(), None);
        log::debug!("New peer connected: {}", peer_id);
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
        log::debug!("Peer disconnected: {}", peer_id);
    }

    pub fn handle_message(&mut self, peer_id: PeerId, message: CryptonoteP2PMessage) {
        match message {
            CryptonoteP2PMessage::Empty => {}
            CryptonoteP2PMessage::Info(node_info) => {
                log::debug!("Info from {}", peer_id);

                if let Some(current_node_info) = self.peers.get_mut(&peer_id) {
                    *current_node_info = Some(node_info);
                }
            }
            CryptonoteP2PMessage::Blocks(blocks) => {
                let mut core = self.core.write().unwrap();
                let blockchain = core.blockchain_mut();

                for block in blocks {
                    if blockchain.get_block(&block.get_hash()).is_none() {
                        blockchain.add_new_block(block).unwrap();
                    }
                }
            }
            CryptonoteP2PMessage::Transactions(_transactions) => unimplemented!(),
            CryptonoteP2PMessage::GetInfo => {
                log::debug!("GetInfo from {}", peer_id);
                let core = self.core.read().unwrap();
                let blockchain = core.blockchain();

                let node_info = NodeInfo {
                    chain_height: blockchain.get_tail().unwrap().0,
                };

                drop(core);

                self.send_event(peer_id, CryptonoteP2PMessage::Info(node_info));
            }
            CryptonoteP2PMessage::GetBlocks(start, end) => {
                let core = self.core.read().unwrap();
                let blockchain = core.blockchain();

                // TODO: Implement alternative chain block retrieval
                let blocks = blockchain.get_blocks(start, end);

                drop(core);

                self.send_event(peer_id, CryptonoteP2PMessage::Blocks(blocks));
            }
            CryptonoteP2PMessage::GetTransactions(txids) => {
                // TODO: Implement unconfirmed transaction retrieval
                let core = self.core.read().unwrap();
                let blockchain = core.blockchain();

                let transactions = txids
                    .iter()
                    .filter_map(|txid| blockchain.get_transaction(txid))
                    .collect::<Vec<_>>();

                drop(core);

                self.send_event(peer_id, CryptonoteP2PMessage::Transactions(transactions));
            }
        }
    }
}

impl<TCoin> Stream for CryptonoteP2PHandler<TCoin>
where
    TCoin: EmissionCurve,
{
    type Item = (PeerId, CryptonoteP2PMessage);
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        {
            let mut core = self.core.write().unwrap();
            let blockchain = core.blockchain_mut();

            if let Ok(Async::Ready(Some(block))) = blockchain.poll() {
                for (peer_id, _) in self.peers.iter() {
                    self.pending_messages.push_back((
                        peer_id.clone(),
                        CryptonoteP2PMessage::Blocks(vec![block.clone()]),
                    ));
                }
            }
        }

        if let Some(message) = self.pending_messages.pop_front() {
            return Ok(Async::Ready(Some(message)));
        }
        Ok(Async::NotReady)
    }
}
