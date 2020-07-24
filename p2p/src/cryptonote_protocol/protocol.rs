use std::future::Future;
use std::pin::Pin;

use anyhow::Context;
use libp2p::{
    core::upgrade::{self, InboundUpgrade, OutboundUpgrade, UpgradeInfo},
    futures::io::{AsyncRead, AsyncWrite},
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

impl From<()> for CryptonoteP2PUpgrade {
    fn from(_: ()) -> Self {
        CryptonoteP2PUpgrade(CryptonoteP2PMessage::Empty)
    }
}

impl From<CryptonoteP2PMessage> for CryptonoteP2PUpgrade {
    fn from(message: CryptonoteP2PMessage) -> Self {
        CryptonoteP2PUpgrade(message)
    }
}

/// Handles sending the actual message to the network
#[derive(Clone)]
pub struct CryptonoteP2PUpgrade(pub CryptonoteP2PMessage);

impl UpgradeInfo for CryptonoteP2PUpgrade {
    type Info = &'static [u8];
    type InfoIter = std::iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        std::iter::once(b"/unprll/1.0.0")
    }
}

type UpgradeFuture<Output, Error> = Pin<Box<dyn Future<Output = Result<Output, Error>> + Send>>;

impl<TSocket> InboundUpgrade<TSocket> for CryptonoteP2PUpgrade
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = CryptonoteP2PMessage;
    type Error = anyhow::Error;
    type Future = UpgradeFuture<Self::Output, Self::Error>;

    fn upgrade_inbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        Box::pin(async move {
            let packet = upgrade::read_one(&mut socket, 1_048_576).await?;
            bincode::deserialize(&packet).with_context(|| "Error deserializing incoming packet")
        })
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for CryptonoteP2PUpgrade
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = ();
    type Error = std::io::Error;
    type Future = UpgradeFuture<Self::Output, Self::Error>;

    fn upgrade_outbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        Box::pin(async move {
            let packet =
                bincode::serialize(&self.0).map_err(|_| std::io::ErrorKind::InvalidInput)?;
            upgrade::write_one(&mut socket, packet).await?;
            Ok(())
        })
    }
}
