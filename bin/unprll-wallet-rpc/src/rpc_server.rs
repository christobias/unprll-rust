use std::sync::{Arc, RwLock};

use jsonrpc_v2::{Data, Error, MapRouter, Params, Server};

use crypto::KeyPair;
use wallet::{SubAddressIndex, Wallet};

use crate::{api_definitions::*, wallet_store::WalletStore};

type WalletStoreRef = Arc<RwLock<WalletStore>>;

pub fn build_server(wallet_store_ref: WalletStoreRef) -> Result<Arc<Server<MapRouter>>, Error> {
    let s = Server::new()
        .with_data(Data::new(wallet_store_ref))
        // Wallet create/load ops
        .with_method("create_wallet", create_wallet)
        .with_method("load_wallet", load_wallet)
        .with_method("refresh_wallets", refresh_wallets)
        .with_method("save_wallets", save_wallets)
        // (Sub)Address management
        .with_method("get_addresses", get_addresses)
        .with_method("get_balances", get_balances)
        .finish();

    Ok(s)
}

async fn create_wallet(
    Params(params): Params<CreateWalletRequest>,
    data: Data<WalletStoreRef>,
) -> Result<(), Error> {
    let spend_keypair = KeyPair::generate();

    let w = Wallet::from(spend_keypair.secret_key);
    data.write()
        .unwrap()
        .add_wallet(params.wallet_name, w)
        .map_err(Error::from)
}

async fn load_wallet(
    Params(params): Params<LoadWalletRequest>,
    data: Data<WalletStoreRef>,
) -> Result<(), Error> {
    let mut data = data.write().unwrap();

    data.load_wallet(params.wallet_name).map_err(Error::from)
}

async fn refresh_wallets(data: Data<WalletStoreRef>) -> Result<(), Error> {
    let data = data.write().unwrap();

    // TODO:
    let _tmp = data.refresh_wallets();
    Ok(())
}

async fn save_wallets(data: Data<WalletStoreRef>) -> Result<(), Error> {
    let data = data.write().unwrap();

    data.save_wallets().map_err(Error::from)
}

async fn get_addresses(
    Params(params): Params<GetAddressesRequest>,
    data: Data<WalletStoreRef>,
) -> Result<GetAddressesResponse, Error> {
    let wallet_store = data.read().unwrap();

    let major_index = params.account_index;
    let minor_indices = params.address_indices.unwrap_or_else(|| vec![{ 0 }]);
    let mut response = GetAddressesResponse::default();

    {
        let wallet = wallet_store.get_wallet(&params.wallet_name)?;
        let wallet = wallet.read().unwrap();

        for index in minor_indices {
            let address = wallet.get_address_for_index(&SubAddressIndex(major_index, index));

            response.addresses.insert(index, address.into());
        }
    }
    Ok(response)
}

async fn get_balances(
    Params(params): Params<GetBalancesRequest>,
    data: Data<WalletStoreRef>,
) -> Result<GetBalancesResponse, Error> {
    let wallet_store = data.read().unwrap();

    let mut response = GetBalancesResponse::default();

    let wallet = wallet_store.get_wallet(&params.wallet_name)?;
    let wallet = wallet.read().unwrap();

    for major_index in params.account_indices {
        response.balances.insert(
            major_index,
            wallet
                .get_account(major_index)
                .ok_or_else(|| {
                    Error::from(failure::format_err!(
                        "Account at index {} does not exist",
                        major_index
                    ))
                })?
                .balance(),
        );
    }
    Ok(response)
}
