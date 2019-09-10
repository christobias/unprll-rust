use std::time::Duration;
use std::collections::{
    HashSet,
    VecDeque
};
use std::sync::{
    Arc,
    RwLock
};

use futures::{
    Async,
    Stream
};
use libp2p::{
    core::{
        ConnectedPoint,
        PeerId
    },
    Multiaddr,
    swarm::{
        NetworkBehaviour,
        NetworkBehaviourAction,
        protocols_handler::{
            OneShotHandler,
            SubstreamProtocol
        },
        PollParameters
    },
    tokio_io::{
        AsyncRead,
        AsyncWrite
    }
};
use log::debug;

use common::GetHash;
use cryptonote_core::CryptonoteCore;

use super::protocol::{
    CryptonoteP2PUpgrade,
    CryptonoteP2PMessage
};

// IDEA: Further split each component into its own parts for easier use by other coins

/// `NetworkBehaviour` to drive the Cryptonote P2P protocol
pub struct CryptonoteNetworkBehavior<TSubstream> {
    core: Arc<RwLock<CryptonoteCore>>,
    events: VecDeque<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PMessage>>,
    marker: std::marker::PhantomData<TSubstream>,
    peers: HashSet<PeerId>
}

impl<TSubstream> CryptonoteNetworkBehavior<TSubstream> {
    pub fn new(_peer_id: PeerId, core: Arc<RwLock<CryptonoteCore>>) -> Self {
        Self {
            core,
            events: VecDeque::new(),
            peers: HashSet::new(),
            marker: std::marker::PhantomData
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviour for CryptonoteNetworkBehavior<TSubstream> {
    type ProtocolsHandler = OneShotHandler<TSubstream, CryptonoteP2PUpgrade, CryptonoteP2PUpgrade, CryptonoteP2PMessage>;
    type OutEvent = CryptonoteP2PMessage;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        OneShotHandler::new(
            SubstreamProtocol::from(CryptonoteP2PUpgrade { message: CryptonoteP2PMessage::Empty }),
            Duration::from_secs(10)
        )
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, endpoint: ConnectedPoint) {
        debug!("New node connected: {} {:?}", peer_id, endpoint);
        self.peers.insert(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, endpoint: ConnectedPoint) {
        debug!("Node disconnected: {} {:?}", peer_id, endpoint);
        self.peers.remove(peer_id);
    }

    fn inject_node_event(&mut self, _peer_id: PeerId, event: CryptonoteP2PMessage) {
        match event {
            CryptonoteP2PMessage::NewBlock(block) => {
                let mut core = self.core.write().unwrap();
                let blockchain = core.blockchain_mut();
                if blockchain.get_block(&block.get_hash()).is_none() {
                    blockchain.add_new_block(*block).unwrap();
                }
            },
            CryptonoteP2PMessage::Empty => {}
        }
    }

    fn poll(&mut self, _params: &mut impl PollParameters) -> Async<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PMessage>> {
        if let Ok(Async::Ready(Some(block))) = self.core.write().unwrap().blockchain_mut().poll() {
            for peer_id in self.peers.iter().cloned() {
                self.events.push_back(NetworkBehaviourAction::SendEvent {
                    event: CryptonoteP2PUpgrade {
                        message: CryptonoteP2PMessage::NewBlock(Box::from(block.clone()))
                    },
                    peer_id
                })
            }
        }

        if let Some(event) = self.events.pop_front() {
            return Async::Ready(event);
        }
        Async::NotReady
    }
}
