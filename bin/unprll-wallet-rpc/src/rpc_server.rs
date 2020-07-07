use std::sync::{Arc, RwLock};

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
                WalletRPC::CreateWallet {
                    respond,
                    wallet_name,
                    ..
                } => {
                    let spend_keypair = KeyPair::generate();

                    let w = Wallet::from_spend_secret_key(spend_keypair.secret_key);

                    match self
                        .wallet_store
                        .write()
                        .unwrap()
                        .add_wallet(wallet_name.clone(), w)
                    {
                        Ok(()) => respond.ok("").await,
                        Err(error) => {
                            respond.err(Error::invalid_params(error.to_string())).await;
                        }
                    };
                }

                WalletRPC::LoadWallet {
                    respond,
                    wallet_name,
                    ..
                } => {
                    match self
                        .wallet_store
                        .write()
                        .unwrap()
                        .load_wallet(wallet_name.clone())
                    {
                        Ok(()) => respond.ok("").await,
                        Err(error) => {
                            respond.err(Error::invalid_params(error.to_string())).await;
                        }
                    };
                }

                WalletRPC::RefreshWallets { respond } => {
                    let response =
                        async { self.wallet_store.write().unwrap().refresh_wallets().await };

                    match response.await {
                        Ok(()) => respond.ok("").await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }

                WalletRPC::SaveWallets {} => {
                    self.wallet_store
                        .write()
                        .unwrap()
                        .save_wallets()
                        .unwrap_or_else(|_| {});
                }

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
                            .ok_or_else(|| failure::format_err!("Wallet not found"))?;
                        let wallet = wallet.read().unwrap();

                        for index in minor_indices {
                            let address =
                                wallet.get_address_for_index(&SubAddressIndex(major_index, index));

                            response.addresses.insert(
                                index,
                                address.to_address_string::<coin_specific::Unprll>(),
                            );
                        }
                        Ok::<_, failure::Error>(response)
                    };

                    match response.await {
                        Ok(response) => respond.ok(response).await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }

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
                            .ok_or_else(|| failure::format_err!("Wallet not found"))?;
                        let wallet = wallet.read().unwrap();
                        for major_index in account_indices {
                            response.balances.insert(
                                major_index,
                                wallet
                                    .get_account(major_index)
                                    .ok_or_else(|| {
                                        failure::format_err!(
                                            "Account at index {} does not exist",
                                            major_index
                                        )
                                    })?
                                    .balance(),
                            );
                        }
                        Ok::<_, failure::Error>(response)
                    };

                    match response.await {
                        Ok(response) => respond.ok(response).await,
                        Err(error) => respond.err(Error::invalid_params(error.to_string())).await,
                    };
                }
            }
        }
    }
}
