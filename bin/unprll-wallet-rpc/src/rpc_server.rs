use std::sync::{
    Arc,
    RwLock
};

use jsonrpc_v2::{
    Error,
    Params,
    Server,
    State
};

use crypto::KeyPair;
use wallet::{
    SubAddressIndex,
    Wallet
};

use crate::{
    api_definitions::*,
    wallet_store::WalletStore
};

type WalletStoreRef = Arc<RwLock<WalletStore>>;

pub fn build_server(wallet_store_ref: WalletStoreRef) -> Result<Server<WalletStoreRef>, Error> {
    let s = Server::with_state(wallet_store_ref)
        // Wallet create/load ops
        .with_method("create_wallet", create_wallet)
        .with_method("load_wallet", load_wallet)
        .with_method("save_wallets", save_wallets)

        // (Sub)Address management
        .with_method("get_addresses", get_addresses)
        .finish();

    Ok(s)
}

fn create_wallet(Params(params): Params<CreateWalletRequest>, state: State<WalletStoreRef>) -> Result<(), Error> {
    let spend_keypair = KeyPair::generate();

    let w = Wallet::from(spend_keypair.secret_key);
    state.write().unwrap().add_wallet(params.wallet_name, w).map_err(Error::from)
}

fn load_wallet(Params(params): Params<LoadWalletRequest>, state: State<WalletStoreRef>) -> Result<(), Error> {
    let mut state = state.write().unwrap();

    state.load_wallet(params.wallet_name).map_err(jsonrpc_v2::Error::from)
}

fn save_wallets(state: State<WalletStoreRef>) -> Result<(), Error> {
    let state = state.write().unwrap();

    state.save_wallets().map_err(jsonrpc_v2::Error::from)
}

fn get_addresses(Params(params): Params<GetAddressesRequest>, state: State<WalletStoreRef>) -> Result<GetAddressesResponse, Error> {
    let wallet_store = state.read().unwrap();

    let major_index = params.account_index;
    let minor_indices = params.address_indices.unwrap_or_else(|| vec!({0}));
    let mut response = GetAddressesResponse::default();

    {
        let wallet = wallet_store.get_wallet(&params.wallet_name)?;
        let wallet = wallet.read().unwrap();

        for index in minor_indices {
            let address = wallet.get_address_for_index(&SubAddressIndex(major_index, index));

            response.addresses.insert(
                index,
                address.into()
            );
        }
    }
    Ok(response)
}
        );
    }
    Ok(response)
}
