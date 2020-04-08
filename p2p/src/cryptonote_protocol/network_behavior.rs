//! # Libp2p specific code
//!
//! Handles taking events from Libp2p and passing them to our code (for better code organization)

use std::{
    future::Future,
    sync::{Arc, RwLock},
    task::{Context, Poll},
    time::Duration,
};

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

use super::{
    protocol::{CryptonoteP2PMessage, CryptonoteP2PUpgrade},
    protocol_handler::CryptonoteP2PHandler,
};

// IDEA: Further split each component into its own parts for easier use by other coins

/// `NetworkBehaviour` to drive the Cryptonote P2P protocol
pub struct CryptonoteNetworkBehavior<TCoin>
where
    TCoin: EmissionCurve + Unpin,
{
    handler: CryptonoteP2PHandler<TCoin>,
}

impl<TCoin> CryptonoteNetworkBehavior<TCoin>
where
    TCoin: EmissionCurve + Unpin,
{
    pub fn new(_peer_id: PeerId, core: Arc<RwLock<CryptonoteCore<TCoin>>>) -> Self {
        Self {
            handler: CryptonoteP2PHandler::new(core),
        }
    }
}

/// Interfacing code with libp2p
impl<TCoin> NetworkBehaviour for CryptonoteNetworkBehavior<TCoin>
where
    TCoin: cryptonote_core::EmissionCurve + Unpin + Send + Sync + 'static,
{
    type ProtocolsHandler =
        OneShotHandler<CryptonoteP2PUpgrade, CryptonoteP2PUpgrade, CryptonoteP2PMessage>;
    type OutEvent = CryptonoteP2PMessage;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        OneShotHandler::new(
            SubstreamProtocol::from(CryptonoteP2PUpgrade {
                message: CryptonoteP2PMessage::Empty,
            }),
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
        self.handler.add_new_peer(peer_id);
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        self.handler.remove_peer(peer_id);
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        _connection_id: ConnectionId,
        event: CryptonoteP2PMessage,
    ) {
        self.handler.handle_message(peer_id, event);
    }

    fn poll(
        &mut self,
        context: &mut Context,
        _params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<CryptonoteP2PUpgrade, CryptonoteP2PMessage>> {
        let event = (&mut self.handler).next();
        futures::pin_mut!(event);

        if let Poll::Ready(Some((peer_id, message))) = event.poll(context) {
            return Poll::Ready(NetworkBehaviourAction::NotifyHandler {
                event: CryptonoteP2PUpgrade { message },
                handler: NotifyHandler::Any,
                peer_id,
            });
        }
        Poll::Pending
    }
}
