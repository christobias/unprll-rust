use futures::future::Future;
use log::{
    error,
    info
};
use structopt::StructOpt;

mod api_definitions;
mod config;
mod rpc_server;
mod wallet_store;

pub use config::Config;

fn main() {
    let config = Config::from_args();
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse().unwrap();
    bin_common::logger::init(&config.bin_common_config, "unprll-wallet-rpc").unwrap();

    let server = hyper::Server::bind(&addr)
        .serve(
            rpc_server::build_server()
                .map_err(|_| error!("Failed to start RPC server!"))
                .unwrap()
                .into_web_service()
        )
        .map_err(|e| error!("server error: {}", e));

    info!("RPC server listening on {}", addr);
    tokio::run(server);
}
