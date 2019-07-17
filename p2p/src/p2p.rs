use std::sync::RwLock;

use libp2p::PeerId;
use libp2p::Swarm;
use libp2p::identity::Keypair;
use libp2p::multiaddr::{Multiaddr, Protocol};
use tokio::prelude::*;
use tokio::runtime::Runtime;

use common::Config;
use cryptonote_core::CryptonoteCore;

use crate::net_behavior::CryptonoteNetworkBehavior;

pub struct P2P {
    bind_port: u16,
    local_key: Keypair,
    peer_id: PeerId,
    core: RwLock<CryptonoteCore>
}

impl P2P {
    pub fn new(config: &Config, core: RwLock<CryptonoteCore>) -> Self {
        // Create a random PeerId
        let local_key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        P2P {
            bind_port: config.p2p_bind_port,
            local_key,
            peer_id,
            core
        }
    }
    pub fn init_server(self, runtime: &mut Runtime) -> Result<(), std::io::Error> {
        // Set up the swarm
        let mut swarm = {
            let transport = libp2p::build_development_transport(self.local_key);
            let network_behavior = CryptonoteNetworkBehavior::new(self.peer_id.clone(), self.core);
            Swarm::new(transport, network_behavior, self.peer_id)
        };

        // Get which address to listen to
        let addr = {
            let mut m = Multiaddr::empty();
            m.push(Protocol::Ip4("0.0.0.0".parse().unwrap()));
            m.push(Protocol::Tcp(self.bind_port));
            m
        };

        Swarm::listen_on(&mut swarm, addr).unwrap();

        runtime.spawn(futures::future::poll_fn(move || -> Result<_, ()> {
            loop {
                match swarm.poll().expect("Error while polling swarm") {
                    Async::Ready(Some(_)) => {},
                    Async::Ready(None) | Async::NotReady => {
                        if let Some(a) = Swarm::listeners(&swarm).next() {
                                println!("Listening on {:?}", a);
                        }
                        break
                    }
                }
            }
            Ok(Async::NotReady)
        }));
        Ok(())
    }
}
