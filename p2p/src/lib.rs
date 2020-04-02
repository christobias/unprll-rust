use std::sync::{Arc, RwLock};

use libp2p::{
    identity::Keypair,
    multiaddr::{Multiaddr, Protocol},
    PeerId, Swarm,
};
use log::info;
use tokio::{prelude::*, runtime::Runtime};

use cryptonote_core::{CryptonoteCore, EmissionCurve};

mod config;
mod cryptonote_protocol;

pub use config::Config;
use cryptonote_protocol::CryptonoteNetworkBehavior;

pub fn init<TCoin: 'static + EmissionCurve + Send + Sync>(
    config: &Config,
    runtime: &mut Runtime,
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
) -> Result<(), std::io::Error> {
    // Create a random PeerId
    let local_key = Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());

    // Set up the swarm
    let mut swarm = {
        let transport = libp2p::build_development_transport(local_key);
        let network_behavior = CryptonoteNetworkBehavior::new(peer_id.clone(), core);
        Swarm::new(transport, network_behavior, peer_id)
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

    runtime.spawn(swarm.into_future().map(|_| {}).map_err(|_| {}));

    info!("P2P server listening on {}", addr);
    Ok(())
}
