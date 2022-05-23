mod middleware;
mod tls;
mod monitor;

use hyper::{Body, Client, Error, Server};

use std::io;
use std::env;
use hyper::server::conn::AddrIncoming;
use futures::join;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use middleware::route::RouteLayer;
use tls::tls_acceptor::{ TlsAcceptor, make_tls_config };
use tls::tls_connector::make_http_or_https_client;

use sysinfo::{System, SystemExt};
use monitor::system::{ get_cpu_usage, get_memory_usage, get_network_usage };

const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

#[tokio::main]
async fn main() {

    tokio::spawn(monitoring());

    // proxy_client for clone()
    let client_main = make_http_or_https_client();
    let client = client_main.clone();

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    let service = ServiceBuilder::new()
        .layer(RouteLayer::new())
        .service_fn(move |mut req: Request<Body>| {
            println!("proxy!, {}", req.uri());      
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


async fn monitoring() -> Result<(), io::Error> {
    use std::time::Duration;
    use cpu_monitor::CpuInstant;

    // monitoring info
    let mut start = CpuInstant::now()?;
    let mut my_system = System::new_all();
    my_system.refresh_all();

    loop {
        println!("----------------------for admin (5 sleep)-----------------------");
        // 1. memory usage
        let (memory_usage, memory_total) = get_memory_usage(&my_system);
        println!("memory usage = {}/{} KB", memory_usage, memory_total);
        
        // 2. network usage
        let (network_in, network_out) = get_network_usage(&my_system);
        println!("network usage = {}/{} KB", network_in, network_out);

        // 3. cpu usage
        let (end, cpu_usage) = get_cpu_usage(start).unwrap();
        println!("cpu usage = {:.0} %", cpu_usage);
        start = end;

        my_system.refresh_all();
        std::thread::sleep(Duration::from_millis(5000));
    }
}