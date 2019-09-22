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
        .with_method("create_wallet", create_wallet)
        .finish();

    Ok(s)
}

fn create_wallet(Params(params): Params<CreateWalletRequest>, state: State<WalletStoreRef>) -> Result<(), Error> {
    let spend_keypair = KeyPair::generate();

    let w = Wallet::from(spend_keypair.secret_key);
    state.write().unwrap().add_wallet(params.file_name, w).map_err(Error::from)
}
