//! # Libp2p specific code
//!
//! Handles taking events from Libp2p and passing them to our code (for better code organization)

use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    sync::{Arc, RwLock},
    task::{Context, Poll},
    time::Duration,
};

use common::GetHash;
use cryptonote_core::{CryptonoteCore, EmissionCurve};
use futures::StreamExt;
use libp2p::{
    core::{connection::ConnectionId, PeerId},
    swarm::{
        protocols_handler::{OneShotHandler, OneShotHandlerConfig, SubstreamProtocol},
        NetworkBehaviour, NetworkBehaviourAction, NotifyHandler, PollParameters,
    },
    Multiaddr,
};

use super::protocol::{CryptonoteP2PMessage, CryptonoteP2PUpgrade, NodeInfo};

// IDEA: Further split each component into its own parts for easier use by other coins

/// `NetworkBehaviour` to drive the Cryptonote P2P protocol
pub struct CryptonoteNetworkBehavior<TCoin>
where
    TCoin: EmissionCurve + Unpin,
{
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
    peers: HashMap<PeerId, Option<NodeInfo>>,
    // It's an ArcRwLock to bypass mut issues
    pending_messages: VecDeque<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PUpgrade>>,
}

impl<TCoin> CryptonoteNetworkBehavior<TCoin>
where
    TCoin: EmissionCurve + Unpin,
{
    pub fn new(_peer_id: PeerId, core: Arc<RwLock<CryptonoteCore<TCoin>>>) -> Self {
        Self {
            core,
            peers: HashMap::new(),
            pending_messages: VecDeque::new(),
        }
    }

    fn send_message(&mut self, connection_id: ConnectionId, peer_id: PeerId, message: CryptonoteP2PMessage) {
        self.pending_messages.push_back(NetworkBehaviourAction::NotifyHandler {
            event: CryptonoteP2PUpgrade(message),
            handler: NotifyHandler::One(connection_id),
            peer_id
        });
    }

    fn handle_message(&mut self, peer_id: PeerId, connection_id: ConnectionId, message: CryptonoteP2PMessage) {
        match message {
            CryptonoteP2PMessage::Empty => {}
            CryptonoteP2PMessage::Info(node_info) => {
                log::debug!("Info from {}", peer_id);

                if let Some(current_node_info) = self.peers.get_mut(&peer_id) {
                    *current_node_info = Some(node_info.clone());

                    // Start syncing from this node if we're lagging behind
                    let core = self.core.read().unwrap();
                    let (current_height, _) = core.blockchain().get_tail().unwrap();

                    drop(core);

                    if current_height < node_info.chain_height {
                        log::info!(
                            "Syncing from {}. Current Height: {}, Target height: {}",
                            peer_id,
                            current_height,
                            node_info.chain_height
                        );
                        self.send_message(connection_id, peer_id, CryptonoteP2PMessage::GetBlocks(current_height + 1, current_height + 20))
                    }
                }
            }
            CryptonoteP2PMessage::Blocks(blocks) => {
                let mut core = self.core.write().unwrap();
                let blockchain = core.blockchain_mut();

                let had_blocks = !blocks.is_empty();

                for block in blocks {
                    if blockchain.get_block(&block.get_hash()).is_none() {
                        // TODO: Drop connection if block was invalid (alt chain blocks allowed)
                        blockchain.add_new_block(block).unwrap_or(());
                    }
                }

                // If we're syncing, send the request for the next block range
                if had_blocks {
                    let (current_height, _) = blockchain.get_tail().unwrap();

                    drop(core);

                    self.send_message(connection_id, peer_id, CryptonoteP2PMessage::GetBlocks(current_height + 1, current_height + 20));
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

                self.send_message(connection_id, peer_id, CryptonoteP2PMessage::Info(node_info));
            }
            CryptonoteP2PMessage::GetBlocks(start, end) => {
                let core = self.core.read().unwrap();
                let blockchain = core.blockchain();

                // TODO: Implement alternative chain block retrieval
                let blocks = blockchain.get_blocks(start, end);

                drop(core);

                self.send_message(connection_id, peer_id, CryptonoteP2PMessage::Blocks(blocks));
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

                self.send_message(connection_id, peer_id, CryptonoteP2PMessage::Transactions(transactions));
            }
        }
    }
}

/// Interfacing code with libp2p
impl<TCoin> NetworkBehaviour for CryptonoteNetworkBehavior<TCoin>
where
    TCoin: cryptonote_core::EmissionCurve + Unpin + Send + Sync + 'static,
{
    type ProtocolsHandler =
        OneShotHandler<CryptonoteP2PUpgrade, CryptonoteP2PUpgrade, CryptonoteP2PUpgrade>;
    type OutEvent = CryptonoteP2PUpgrade;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        OneShotHandler::new(
            SubstreamProtocol::from(CryptonoteP2PUpgrade(CryptonoteP2PMessage::Empty)),
            OneShotHandlerConfig {
                inactive_timeout: Duration::from_secs(600),
                substream_timeout: Duration::from_secs(608),
            },
        )
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        self.pending_messages.push_back(NetworkBehaviourAction::NotifyHandler {
            event: CryptonoteP2PUpgrade(CryptonoteP2PMessage::GetInfo),
            handler: NotifyHandler::Any,
            peer_id: peer_id.clone()
        });
        self.peers.insert(peer_id.clone(), None);
        log::debug!("New peer connected: {}", peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
        log::debug!("Peer disconnected: {}", peer_id);
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: CryptonoteP2PUpgrade,
    ) {
        self.handle_message(peer_id, connection_id, event.0);
    }

    fn poll(
        &mut self,
        context: &mut Context,
        _params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PUpgrade>> {
        {
            let mut core = self.core.write().unwrap();
            let blockchain = core.blockchain_mut();

            let block = blockchain.next();
            futures::pin_mut!(block);

            // Check for new blocks from the blockchain
            // TODO FIXME: Blocking on a future feels incorrect within an async context
            if let Poll::Ready(Some(block)) = block.poll(context) {
                for (peer_id, _) in self.peers.iter() {
                    self.pending_messages.push_back(NetworkBehaviourAction::NotifyHandler {
                        event: CryptonoteP2PUpgrade(CryptonoteP2PMessage::Blocks(vec![block.clone()])),
                        handler: NotifyHandler::Any,
                        peer_id: peer_id.clone(),
                    });
                }
            }
        }

        if let Some(message) = self.pending_messages.pop_front() {
            return Poll::Ready(message);
        }
        Poll::Pending
    }
}
