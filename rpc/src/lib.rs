#[macro_use] extern crate log;

use std::sync::{Arc, RwLock};

use futures::future::Future;
use hyper::{
    Body,
    Method,
    Request,
    Response,
    service::{
        make_service_fn,
        service_fn
    },
    Server
};
use tokio::{
    runtime::Runtime
};

use common::Config;
use cryptonote_core::CryptonoteCore;

pub fn init(config: &Config, runtime: &mut Runtime, core: Arc<RwLock<CryptonoteCore>>) {
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse().unwrap();

    // Hook up the router to the hyper server
    let server = Server::bind(&addr).serve(make_service_fn(move |_| {
        let core = core.clone();
        service_fn(move |req| {
            Ok::<_, hyper::Error>(router(req, core.clone()))
        })
    })).map_err(|_| {});

    runtime.spawn(server);
    info!("RPC server listening on {}", addr);
}

fn router(req: Request<Body>, core: Arc<RwLock<CryptonoteCore>>) -> Response<Body> {
    let mut response = Response::builder();
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => response.body(Body::from("Hello, world!")),
        _ => response.status(404).body(Body::from("Endpoint not found"))
    };
    response.unwrap()
}
