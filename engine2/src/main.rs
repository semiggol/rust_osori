mod middleware;
mod tls;
mod admin;
mod monitor;
mod config;

use hyper::{Body, Server};
use futures::join;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use middleware::route::RouteLayer;
use middleware::access_log::{AccessLogLayer, AccessLogRequestBody};
use tls::tls_connector::make_http_or_https_client;

// for api map
use admin::apis;

const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

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
    if let Err(e)  = admin::register::register_to_admin(config).await {
        println!("error occurred: {}", e);
        //std::process::exit(-1);
    } else {
        println!("Success to register!");
    }

    // test for global variable
    tokio::spawn(apis::test_update_apis());
    tokio::spawn(apis::test_find_apis());

    // proxy_client for clone()
    let client_main = make_http_or_https_client();
    let client = client_main.clone();

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    let service = ServiceBuilder::new()
        .layer(RouteLayer::new())
        .layer(AccessLogLayer::new())
        .service_fn(move |mut req: Request<AccessLogRequestBody<Body>>| {
            println!("proxy!, {}", req.uri());      
            *req.uri_mut() = API_SAMPLE_DOMAIN.parse().unwrap();
            client.request(req.map(|inner| inner.inner))
        });

    // http server
    let http_server = Server::bind(&http_addr).serve(Shared::new(service));


    println!("Listening on http://{}", http_addr);
    println!("Proxying on {}", API_SAMPLE_DOMAIN);

    if let Err(e) = http_server.await {
        eprintln!("server error: {}", e);
    }
}
