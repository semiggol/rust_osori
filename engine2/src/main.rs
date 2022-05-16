mod middleware;
mod tls;

use hyper::{Body, Client, Error, Server};

use std::env;
use hyper::server::conn::AddrIncoming;
use futures::join;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use middleware::route::RouteLayer;
use tls::tls_acceptor::{ TlsAcceptor, make_tls_config };
use tls::tls_connector::make_http_or_https_client;


const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

#[tokio::main]
async fn main() {
    // proxy_client for clone()
    let client_main = make_http_or_https_client();

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    let service = ServiceBuilder::new()
        .layer(RouteLayer::new())
        .service_fn(move |mut req: Request<Body>| {
            println!("proxy!, {}", req.uri());
            let client = client_main.clone();
            *req.uri_mut() = API_SAMPLE_DOMAIN.parse().unwrap();
            client.request(req)
        });

    // http server
    let http_server = Server::bind(&http_addr).serve(Shared::new(service));


    println!("Listening on http://{}", http_addr);
    println!("Proxying on {}", API_SAMPLE_DOMAIN);

    if let Err(e) = http_server.await {
        eprintln!("server error: {}", e);
    }
}
