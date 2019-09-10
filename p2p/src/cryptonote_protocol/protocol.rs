use libp2p::{
    core::upgrade::{
        self,
        InboundUpgrade,
        Negotiated,
        OutboundUpgrade,
        UpgradeInfo
    },
    tokio_io::{
        AsyncRead,
        AsyncWrite
    }
};
use serde::{
    Serialize,
    Deserialize
};

use common::Block;

/// P2P Protocol Messages
#[derive(Clone, Serialize, Deserialize)]
pub enum CryptonoteP2PMessage {
    /// Placeholder for when there is no message
    Empty,

    /// A new block was mined
    NewBlock(Box<Block>)
}

impl From<()> for CryptonoteP2PMessage {
    fn from(_: ()) -> Self {
        CryptonoteP2PMessage::Empty
    }
}

/// Handles sending the actual message to the network
#[derive(Clone)]
pub struct CryptonoteP2PUpgrade {
    /// Message to be sent
    pub message: CryptonoteP2PMessage
}

impl UpgradeInfo for CryptonoteP2PUpgrade {
    type Info = &'static [u8];
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        std::iter::once(b"/unprll/1.0.0")
    }
}

// Callback for read_one_then to keep things readable (recommended by clippy)
type ReadThenCallback = fn(Vec<u8>, ()) -> Result<CryptonoteP2PMessage, upgrade::ReadOneError>;

impl<TSubstream: AsyncRead + AsyncWrite> InboundUpgrade<TSubstream> for CryptonoteP2PUpgrade {
    type Output = CryptonoteP2PMessage;
    type Error = upgrade::ReadOneError;
    type Future = upgrade::ReadOneThen<Negotiated<TSubstream>, (), ReadThenCallback>;

    fn upgrade_inbound(self, socket: Negotiated<TSubstream>, _: Self::Info) -> Self::Future {
        // TODO: Decide on the max packet length
        upgrade::read_one_then(socket, 65536, (), |packet, ()| {
            let message = bincode::deserialize(&packet).unwrap();
            Ok(message)
        })
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> OutboundUpgrade<TSubstream> for CryptonoteP2PUpgrade {
    type Output = ();
    type Error = std::io::Error;
    type Future = upgrade::WriteOne<Negotiated<TSubstream>>;

    fn upgrade_outbound(self, socket: Negotiated<TSubstream>, _: Self::Info) -> Self::Future {
        upgrade::write_one(socket, bincode::serialize(&self.message).unwrap())
    }
}
