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
use wallet::Wallet;

use crate::{
    api_definitions::*,
    wallet_store::WalletStore
};

type WalletStoreRef = Arc<RwLock<WalletStore>>;

pub fn build_server() -> Result<Server<WalletStoreRef>, Error> {
    let s = Server::with_state(Arc::from(RwLock::from(WalletStore::new())))
        // Wallet create/load ops
        .with_method("create_wallet", create_wallet)
        .with_method("load_wallet", load_wallet)

        // (Sub)Address management
        .with_method("get_address", get_address)
        .finish();

    Ok(s)
}

fn create_wallet(Params(params): Params<CreateWalletRequest>, state: State<WalletStoreRef>) -> Result<(), Error> {
    let spend_keypair = KeyPair::generate();

    let w = Wallet::from(spend_keypair.secret_key);
    state.write().unwrap().add_wallet(params.wallet_name, w).map_err(Error::from)
}

fn load_wallet(Params(_params): Params<LoadWalletRequest>, _state: State<WalletStoreRef>) -> Result<(), Error> {
    // TODO: Implement saving wallets to files
    Ok(())
}

fn get_address(Params(params): Params<GetAddressRequest>, state: State<WalletStoreRef>) -> Result<GetAddressResponse, Error> {
    let wallet_store = state.read().unwrap();

    let major_index = params.account_index;
    let minor_indices = params.address_indices.unwrap_or_else(|| vec!({0}));
    let mut response = GetAddressResponse::default();

    for index in minor_indices {
        let wallet = wallet_store.get_wallet(&params.wallet_name)?;
        let address = wallet.get_address_for_index(major_index, index)
            .ok_or_else(|| failure::format_err!("Address at index {} not found", index))?;

        response.addresses.insert(
            index,
            address.into()
        );
    }
    Ok(response)
}
