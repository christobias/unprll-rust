#![deny(missing_docs)]

//! # Cryptonote P2P Networking module

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

use libp2p::{
    core::Executor,
    identity::Keypair,
    multiaddr::{Multiaddr, Protocol},
    swarm::SwarmBuilder,
    PeerId, Swarm,
};
use log::info;

use cryptonote_core::{CryptonoteCore, EmissionCurve};

mod config;
mod cryptonote_protocol;

pub use config::Config;
use cryptonote_protocol::CryptonoteNetworkBehavior;

struct TokioExecutor;

impl Executor for TokioExecutor {
    fn exec(&self, future: Pin<Box<dyn Future<Output = ()> + 'static + Send>>) {
        tokio::spawn(future);
    }
}

/// Initialize the P2P handler
pub fn init<TCoin: 'static + EmissionCurve + Unpin + Send + Sync>(
    config: &Config,
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
) -> Result<impl Future, failure::Error> {
    // Create a random PeerId
    let local_key = Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());

    // Set up the swarm
    let mut swarm = {
        let transport = libp2p::build_development_transport(local_key)?;
        let network_behavior = CryptonoteNetworkBehavior::new(peer_id.clone(), core);
        SwarmBuilder::new(transport, network_behavior, peer_id)
            .executor(Box::from(TokioExecutor))
            .build()
    };

    // Get which address to listen to
    let addr = {
        let mut m = Multiaddr::empty();
        m.push(Protocol::Ip4("0.0.0.0".parse().unwrap()));
        m.push(Protocol::Tcp(config.p2p_bind_port));
        m
    };

    if let Some(peer) = &config.connect_to {
        Swarm::dial_addr(&mut swarm, peer.parse().unwrap()).unwrap();
    }
    Swarm::listen_on(&mut swarm, addr.clone()).unwrap();

    info!("P2P server listening on {}", addr);
    Ok(async move {
        loop {
            // Keep polling the swarm non-stop
            swarm.next().await;
        }
    })
}
