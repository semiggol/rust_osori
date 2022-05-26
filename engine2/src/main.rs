mod middleware;
mod tls;
mod admin;
mod monitor;

use hyper::{Body, Server};
use futures::join;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use middleware::route::RouteLayer;
use middleware::access_log::{AccessLogLayer, AccessLogRequestBody};
use tls::tls_connector::make_http_or_https_client;

// store for api
use std::time::Duration;
use lazy_static::lazy_static;
use dashmap::DashMap;
use admin::apis;

// global variable -> APIS_MAP
lazy_static! {
    static ref APIS_MAP: DashMap<String, apis::Apis> = DashMap::new();
}

const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

#[tokio::main]
async fn main() {
    // register to admin
    if let Err(e)  = admin::register::register_to_admin().await {
        println!("error occurred: {}", e);
        //std::process::exit(-1);
    } else {
        println!("Success to register!");
    }

    // test for global variable: dashmap
    tokio::spawn(test_insert_dashmap()); // insert()
    tokio::spawn(test_get_dashmap()); // get() by other thread

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

async fn test_insert_dashmap() {
    loop {
        let api1 = apis::make_sample_api1();
        let key = api1.get_key();
        APIS_MAP.insert(key, api1);

        let api2 = apis::make_sample_api2();
        let key = api2.get_key();
        APIS_MAP.insert(key, api2);

        // sleep 2 seconds.
        std::thread::sleep(Duration::from_millis(2000));
        // remove data from the map
        APIS_MAP.clear();
        // sleep 2 seconds.
        std::thread::sleep(Duration::from_millis(2000));
    }
}

async fn test_get_dashmap() {
    loop {
        println!("===============test dashmap api ");
        match APIS_MAP.get("/v1/test") {
            Some(api) => {
                println!("test api > {:?}", api.clone());
            },
            None => {
                println!("test api > test Not Found");
            }
        };

        match APIS_MAP.get("/v2/google") {
            Some(api) => {
                println!("test api > {:?}", api.clone());
            },
            None => {
                println!("test api > google Not Found");
            }
        };
        
        // sleep 0.5 seconds.
        std::thread::sleep(Duration::from_millis(500));
    }
}