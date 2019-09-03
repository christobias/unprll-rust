#[macro_use] extern crate libp2p;
#[macro_use] extern crate log;

use libp2p::PeerId;
use libp2p::Swarm;
use libp2p::identity::Keypair;
use libp2p::multiaddr::{Multiaddr, Protocol};
use tokio::prelude::*;
use tokio::runtime::Runtime;

use cryptonote_core::CryptonoteCore;

mod config;
mod net_behavior;

use config::Config;
use net_behavior::CryptonoteNetworkBehavior;

pub fn init(config: &Config, runtime: &mut Runtime, core: CryptonoteCore) -> Result<(), std::io::Error> {
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

    Swarm::listen_on(&mut swarm, addr.clone()).unwrap();

    runtime.spawn(swarm.into_future().then(|_| {
        Ok(())
    }));
    info!("P2P server listening on {}", addr);

    Ok(())
}
