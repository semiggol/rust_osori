mod service;
mod tls;
mod admin;
mod monitor;
mod config;

use hyper::Server;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use service::route::{ RouteLayer, dummy_route };
use service::access_log::{AccessLogLayer, AccessLogRequestBody};
use tls::tls_connector::make_http_or_https_client;

// for api map
use admin::apis;
use crate::service::proxy::ProxyService;

#[tokio::main]
async fn main() {
    let config = match config::parse() {
        Ok(config) => config,
        Err(e) => {
            println!("error occurred: {}", e);
            std::process::exit(-1);
        }
    };

    // register to admin
    if let Err(e)  = admin::register::handle(config).await {
        println!("error occurred: {}", e);
        std::process::exit(-1);
    } else {
        println!("Success to register!");
    }

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    let service = ServiceBuilder::new()
        .layer(RouteLayer::new(dummy_route()))
        .layer(AccessLogLayer::new())
        .service(ProxyService);

    // http server
    let http_server = Server::bind(&http_addr).serve(Shared::new(service));

    println!("Listening on http://{}", http_addr);

    if let Err(e) = http_server.await {
        eprintln!("server error: {}", e);
    }
}
