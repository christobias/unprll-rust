use libp2p::{
    core::upgrade::{self, InboundUpgrade, Negotiated, OutboundUpgrade, UpgradeInfo},
    tokio_io::{AsyncRead, AsyncWrite},
};
use serde::{Deserialize, Serialize};

use common::{Block, Transaction};
use crypto::Hash256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInfo {
    pub chain_height: u64,
}

/// P2P Protocol Messages
#[derive(Clone, Serialize, Deserialize)]
pub enum CryptonoteP2PMessage {
    /// Placeholder for when there is no message
    Empty,

    // ------------- Data Messages -------------
    /// Information about this node, including information about its main
    /// chain
    Info(NodeInfo),

    /// A set of blocks of transactions, as a response to a node sync or
    /// when a miner finds a new block
    Blocks(Vec<Block>),

    /// A set of individual transactions, as a response to a node sync or
    /// when new transactions are broadcasted
    Transactions(Vec<Transaction>),

    // ----------- Request Messages ------------
    /// Request for node info
    GetInfo,

    /// Request for the given blocks from start to end height
    ///
    /// The range of blocks returned is similar to Rust slices (i.e, start inclusive, end exclusive)
    /// The node can send multiple blocks for the same height in case of a chain split, and
    /// fewer blocks than the end in case of shorter node chain height
    GetBlocks(u64, u64),

    /// Request for the given transaction IDs, confirmed and unconfirmed
    ///
    /// If the node does not have all transactions requested, it sends those that
    /// it does have
    GetTransactions(Vec<Hash256>),
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
    pub message: CryptonoteP2PMessage,
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
