//! # Libp2p specific code
//!
//! Handles taking events from Libp2p and passing them to our code (for better code organization)

use std::sync::{Arc, RwLock};
use std::time::Duration;

use futures::{Async, Stream};

use cryptonote_core::{CryptonoteCore, EmissionCurve};
use libp2p::{
    core::{ConnectedPoint, PeerId},
    swarm::{
        protocols_handler::{OneShotHandler, SubstreamProtocol},
        NetworkBehaviour, NetworkBehaviourAction, PollParameters,
    },
    tokio_io::{AsyncRead, AsyncWrite},
    Multiaddr,
};

use super::{
    protocol::{CryptonoteP2PMessage, CryptonoteP2PUpgrade},
    protocol_handler::CryptonoteP2PHandler,
};

// IDEA: Further split each component into its own parts for easier use by other coins

/// `NetworkBehaviour` to drive the Cryptonote P2P protocol
pub struct CryptonoteNetworkBehavior<TCoin, TSubstream>
where
    TCoin: EmissionCurve,
{
    handler: CryptonoteP2PHandler<TCoin>,
    marker: std::marker::PhantomData<TSubstream>,
}

impl<TCoin, TSubstream> CryptonoteNetworkBehavior<TCoin, TSubstream>
where
    TCoin: EmissionCurve,
{
    pub fn new(_peer_id: PeerId, core: Arc<RwLock<CryptonoteCore<TCoin>>>) -> Self {
        Self {
            handler: CryptonoteP2PHandler::new(core),
            marker: std::marker::PhantomData,
        }
    }
}

/// Interfacing code with libp2p
impl<TCoin, TSubstream> NetworkBehaviour for CryptonoteNetworkBehavior<TCoin, TSubstream>
where
    TCoin: cryptonote_core::EmissionCurve,
    TSubstream: AsyncRead + AsyncWrite,
{
    type ProtocolsHandler = OneShotHandler<
        TSubstream,
        CryptonoteP2PUpgrade,
        CryptonoteP2PUpgrade,
        CryptonoteP2PMessage,
    >;
    type OutEvent = CryptonoteP2PMessage;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        OneShotHandler::new(
            SubstreamProtocol::from(CryptonoteP2PUpgrade {
                message: CryptonoteP2PMessage::Empty,
            }),
            Duration::from_secs(600),
        )
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, peer_id: PeerId, _endpoint: ConnectedPoint) {
        self.handler.add_new_peer(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId, _endpoint: ConnectedPoint) {
        self.handler.remove_peer(peer_id);
    }

    fn inject_node_event(&mut self, peer_id: PeerId, event: CryptonoteP2PMessage) {
        self.handler.handle_message(peer_id, event);
    }

    fn poll(
        &mut self,
        _params: &mut impl PollParameters,
    ) -> Async<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PMessage>> {
        if let Ok(Async::Ready(Some((peer_id, message)))) = self.handler.poll() {
            return Async::Ready(NetworkBehaviourAction::SendEvent {
                event: CryptonoteP2PUpgrade { message },
                peer_id,
            });
        }
        Async::NotReady
    }
}
