use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use libp2p::PeerId;
use libp2p::floodsub::{Floodsub, FloodsubEvent, Topic, TopicBuilder};
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::tokio_io::{AsyncRead, AsyncWrite};

use cryptonote_core::CryptonoteCore;

/// Network Behaviour driving the Cryptonote P2P protocol
#[derive(NetworkBehaviour)]
pub struct CryptonoteNetworkBehavior<TSubstream: AsyncRead + AsyncWrite> {
    /// Broadcast messaging with peers
    floodsub: Floodsub<TSubstream>,

    /// Cryptonote Core to interact with the blockchain, transaction pool and other components
    #[behaviour(ignore)]
    core: Arc<RwLock<CryptonoteCore>>,

    /// Topics
    #[behaviour(ignore)]
    topics: HashMap<String, Topic>
}

impl<TSubstream: AsyncRead + AsyncWrite> CryptonoteNetworkBehavior<TSubstream> {
    pub fn new(local_peer_id: PeerId, core: Arc<RwLock<CryptonoteCore>>) -> Self {
        let mut behaviour = CryptonoteNetworkBehavior {
            floodsub: Floodsub::new(local_peer_id),
            core,
            topics: HashMap::new()
        };
        ["blocks", "transactions"].iter().map(|x| x.to_string()).for_each(|topic| {
            behaviour.topics.insert(topic.clone(), TopicBuilder::new(&topic).build());
            behaviour.floodsub.subscribe(behaviour.topics.get(&topic).expect("We just inserted the topic. Absence is fatal").clone());
        });

        behaviour
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<FloodsubEvent> for CryptonoteNetworkBehavior<TSubstream> {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(message) => {
                info!("Messaged received from floodsub: {}", String::from_utf8_lossy(&message.data));
            },
            FloodsubEvent::Subscribed{..} => {},
            FloodsubEvent::Unsubscribed{..} => {}
        }
    }
}
