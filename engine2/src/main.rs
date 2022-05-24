mod middleware;
mod tls;
mod admin;
mod monitor;

use std::io;
use hyper::{Body, Server};
use futures::join;
use http::Request;
use tower::make::Shared;
use tower::ServiceBuilder;
use middleware::route::RouteLayer;
use middleware::access_log::{AccessLogLayer, AccessLogRequestBody};
use tls::tls_connector::make_http_or_https_client;

use monitor::system::{ get_cpu_usage, get_memory_usage, get_network_usage, get_hostname, get_logical_cpus };

const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

#[tokio::main]
async fn main() {
    // register to admin
    if let Err(e)  = admin::register::register_to_admin().await {
        println!("error occurred!{}", e);
        std::process::exit(-1);
    } else {
        println!("Success to register!");
    }

    // sample: system monitoring info
    tokio::spawn(monitoring());

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


// ToDo: Admin will use below info
async fn monitoring() -> Result<(), io::Error> {
    use std::time::Duration;
    use sysinfo::{ System, SystemExt };

    // monitoring info
    let mut my_system = System::new_all();
    my_system.refresh_all();

    // hostname
    let hostname = get_hostname(&my_system);
    
    // logical cpu count
    let cpus = get_logical_cpus(&my_system);
    println!("===> hostname: {}, cpus: {}", hostname, cpus);

    loop {
        println!("----------------------for admin (5 sleep)-----------------------");
        // memory usage
        let (memory_usage, memory_total) = get_memory_usage(&my_system);
        println!("memory usage = {}/{} KB", memory_usage, memory_total);
        
        // network usage
        let (network_in, network_out) = get_network_usage(&my_system);
        println!("network usage = {}/{} KB", network_in, network_out);

        // cpu usage
        let cpu_usage = get_cpu_usage(&my_system);
        println!("cpu usage = {} %", cpu_usage);

        // refresh
        my_system.refresh_all();

        // sleep 5 seconds.
        std::thread::sleep(Duration::from_millis(5000));
    }
}