use std::sync::{Arc, RwLock};

use anyhow::Context;
use jsonrpsee::{common::Error, raw::RawServer, transport::TransportServer};

use crypto::KeyPair;
use transaction_util::subaddress::SubAddressIndex;
use wallet::Wallet;

use crate::{api_definitions::*, wallet_store::WalletStore};

pub struct WalletRPCServer<R, I>
where
    R: TransportServer<RequestId = I>,
    I: Clone + Eq + std::hash::Hash + Send + Sync,
{
    server: Arc<RwLock<RawServer<R, I>>>,
    wallet_store: Arc<RwLock<WalletStore>>,
}

impl<R, I> WalletRPCServer<R, I>
where
    R: TransportServer<RequestId = I>,
    I: Clone + Eq + std::hash::Hash + Send + Sync,
{
    pub fn new(server: RawServer<R, I>, wallet_store: Arc<RwLock<WalletStore>>) -> Self {
        Self {
            server: Arc::from(RwLock::from(server)),
            wallet_store,
        }
    }

    pub async fn run(self) {
        while let Ok(request) = WalletRPC::next_request(&mut self.server.write().unwrap()).await {
            match request {
                // create_wallet
                WalletRPC::CreateWallet {
                    respond,
                    wallet_name,
                    ..
                } => {
                    let spend_keypair = KeyPair::generate();

                    let w = Wallet::from_spend_secret_key(spend_keypair.secret_key);

                    respond.respond(
                        self.wallet_store
                            .write()
                            .unwrap()
                            .add_wallet(wallet_name.clone(), w)
                            .map(|_| "")
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }

                // load_wallet
                WalletRPC::LoadWallet {
                    respond,
                    wallet_name,
                    ..
                } => {
                    respond.respond(
                        self.wallet_store
                            .write()
                            .unwrap()
                            .load_wallet(wallet_name.clone())
                            .map(|_| "")
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }

                // refresh_wallets
                WalletRPC::RefreshWallets { respond } => {
                    respond.respond(
                        self.wallet_store
                            .write()
                            .unwrap()
                            .refresh_wallets()
                            .await
                            .map(|_| "")
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }

                // save_wallets
                WalletRPC::SaveWallets { respond } => {
                    respond.respond(
                        self.wallet_store
                            .write()
                            .unwrap()
                            .save_wallets()
                            .await
                            .map(|_| "")
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }

                // get_addresses
                WalletRPC::GetAddresses {
                    respond,
                    wallet_name,
                    account_index,
                    address_indices,
                } => {
                    let major_index = account_index;
                    let minor_indices = address_indices.unwrap_or_else(|| vec![0]);

                    // Workaround to get ? to work within here. Makes code easier to write
                    let response = async {
                        let mut response = GetAddressesResponse::default();

                        let wallet = self
                            .wallet_store
                            .write()
                            .unwrap()
                            .get_wallet(&wallet_name)
                            .with_context(|| "Wallet not found")?;
                        let wallet = wallet.read().unwrap();

                        for index in minor_indices {
                            let address = wallet
                                .get_address_for_index(&SubAddressIndex(major_index, index))
                                .with_context(|| "Wallet not found")?;

                            response.addresses.insert(
                                index,
                                address.to_address_string::<coin_specific::Unprll>(),
                            );
                        }
                        Ok::<_, anyhow::Error>(response)
                    };

                    respond.respond(
                        response
                            .await
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }

                // get_balances
                WalletRPC::GetBalances {
                    respond,
                    wallet_name,
                    account_indices,
                } => {
                    let response = async {
                        let mut response = GetBalancesResponse::default();

                        let wallet = self
                            .wallet_store
                            .write()
                            .unwrap()
                            .get_wallet(&wallet_name)
                            .with_context(|| "Wallet not found")?;
                        let wallet = wallet.read().unwrap();
                        for major_index in account_indices {
                            response.balances.insert(
                                major_index,
                                wallet
                                    .get_account(major_index)
                                    .with_context(|| "Wallet not found")?
                                    .get_balance(),
                            );
                        }
                        Ok::<_, anyhow::Error>(response)
                    };

                    respond.respond(
                        response
                            .await
                            .map_err(|e| Error::invalid_params(e.to_string())),
                    );
                }
            }
        }
    }
}
