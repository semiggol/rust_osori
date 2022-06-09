mod admin;
mod config;
mod monitor;
mod service;
mod tls;

use crate::service::cors::CorsLayer;
use crate::service::proxy::ProxyService;
use hyper::Server;
use service::access_log::{AccessLogLayer, AccessLogRequestBody};
use service::route::{dummy_route, RouteLayer};
use tls::tls_connector::make_http_or_https_client;
use tower::make::Shared;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let config = match config::args::parse() {
        Ok(config) => config,
        Err(e) => {
            println!("error occurred: {}", e);
            std::process::exit(-1);
        }
    };

    // register to admin
    if let Err(e) = admin::register::handle(config).await {
        println!("error occurred: {}", e);
        std::process::exit(-1);
    } else {
        println!("Success to register!");
    }

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    let service = ServiceBuilder::new()
        .layer(AccessLogLayer::new())
        .layer(RouteLayer::new(dummy_route()))
        .layer(CorsLayer)
        .service(ProxyService);

    // http server
    let http_server = Server::bind(&http_addr).serve(Shared::new(service));

    println!("Listening on http://{}", http_addr);

    if let Err(e) = http_server.await {
        eprintln!("server error: {}", e);
    }
}
