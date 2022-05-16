mod middleware;
mod tls;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error, Server};
use hyper_rustls::{HttpsConnectorBuilder};
// use tower::{ServiceBuilder, make::Shared};

use std::env;
use hyper::server::conn::AddrIncoming;
use futures::join;
// use middleware::route::RouteLayer;
use tls::tls_acceptor::TlsAcceptor;
use tls::tls_acceptor::{ make_tls_config };


const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

#[tokio::main]
async fn main() {
    // http/https client (hyper_rustls)
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    // proxy_client for clone()
    let proxy_client = Client::builder().build::<_, hyper::Body>(https);
    let client1 = proxy_client.clone();
    let client2 = proxy_client.clone();

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    // make_service_fn/service_fn
    let http_service = make_service_fn(move |_conn| {
        println!("osori> http_service: make_service_fn(con={:?})",  _conn);
        let client1 = client1.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                client1.request(req)
            }))
        }
    });

    // http server
    let http_server = Server::bind(&http_addr).serve(http_service);


    // First parameter is port number (optional, defaults to 1337)
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "3003".to_owned(),
    };
    let https_addr = format!("127.0.0.1:{}", port).parse().unwrap();

    // Build TLS configuration. ToDo: use configuration from admin
    let tls_cfg = make_tls_config();

    // Create a TCP listener via tokio.
    let incoming = AddrIncoming::bind(&https_addr).unwrap();
    
    // make_service_fn/service_fn
    let https_service = make_service_fn(move |_conn| {
        println!("osori> https_service: make_service_fn(TLS client)");
        let client2 = client2.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                println!("osori> service_fn(req={:?})",  req.uri());
                client2.request(req)
            }))
        }
    });

    // https server
    let https_server = Server::builder(TlsAcceptor::new(tls_cfg, incoming)).serve(https_service);

    println!("Listening on http://{}", http_addr);
    println!("Listening on https://{}", https_addr);
    println!("Proxying on {}", API_SAMPLE_DOMAIN);

    // Run the future, keep going until an error occurs.
    let (_http_result, _https_result) = join!(http_server, https_server);

}
