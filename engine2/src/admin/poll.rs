use crate::monitor;

use tokio::{task, time};
use std::time::{Duration, UNIX_EPOCH};
use hyper::client::HttpConnector;
use serde_json::Value;
use serde_json::json;
use hyper::{ Client, Request, Method, Body, body, StatusCode };
use std::time::SystemTime;
use monitor::system::{ get_memory_usage, get_network_usage, get_cpu_usage };
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
struct PollMessage {
    id: String,
    time: u128,
    totalMemory: u64,
    usedMemory: u64,
    usedCpu: f32,
    usedNetworkTrafficIn: u64,
    usedNetworkTrafficOut: u64,
    clientCount: usize,
    requestCount: usize,
    responseCount: usize,
    responseTime: usize,
    responseStatus: Vec<usize>,
    activeRequests: Vec<ActiveRequest>,
    errorMessage: String,
}

#[derive(Serialize, Deserialize)]
struct ActiveRequest {
    apiName: String,
    apiVersion: usize,
    elapsedTime: usize,
}

pub fn handle(client: Client<HttpConnector>, info: Value){
    task::spawn( async move {
        let mut interval = time::interval(Duration::from_secs(5));
        let id = info["id"].as_str().unwrap();
        loop {
            interval.tick().await;

            let message = make_poll_message(id);
            send_poll_msg(message, client.clone()).await.unwrap();
        }
    });
}

struct MonitoringInfo {
    memory_usage: u64,
    memory_usage_total: u64,
    network_usage_in: u64,
    network_usage_out: u64,
    cpu_usage: f32,
}

fn get_monitoring_info() -> MonitoringInfo {
    use sysinfo::{ System, SystemExt };

    // monitoring info
    let mut my_system = System::new_all();
    my_system.refresh_all();

    let (memory_usage, memory_usage_total) = get_memory_usage(&my_system);

    // network usage
    let (network_usage_in, network_usage_out) = get_network_usage(&my_system);

    // cpu usage
    let cpu_usage = get_cpu_usage(&my_system);

    MonitoringInfo {
        memory_usage,
        memory_usage_total,
        network_usage_in,
        network_usage_out,
        cpu_usage
    }
}

fn make_poll_message(id: &str) -> String {
    // get the information needed to make a poll message
    let sys_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(e) => {
            panic!("error occurred while getting current time: {}", e);
        }
    };
    let monitoring_info = get_monitoring_info();

    let message = PollMessage {
        id: id.to_string(),
        time: sys_time,
        totalMemory: monitoring_info.memory_usage_total,
        usedMemory: monitoring_info.memory_usage,
        usedCpu: monitoring_info.cpu_usage,
        usedNetworkTrafficIn: monitoring_info.network_usage_in,
        usedNetworkTrafficOut: monitoring_info.network_usage_out,
        clientCount: 0,
        requestCount: 0,
        responseCount: 0,
        responseTime: 0,
        responseStatus: vec![0, 0, 0, 0, 0],
        activeRequests: vec![],
        errorMessage: String::from(""),
    };

    serde_json::to_string(&message).unwrap()
}

async fn send_poll_msg(body: String, client: Client<HttpConnector>) -> Result<(), String> {
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://118.67.135.216:5581/poll")
        .header("content-type", "application/json")
        .body(Body::from(body)).unwrap();

    let resp = match client.request(req).await {
        Ok(resp)  => resp,
        Err(e) => {
            return Err(e.message().to_string())
        }
    };

    if resp.status() != StatusCode::OK {
        return Err(format!("Not 200 OK(status code:{}", resp.status()));
    }

    let body_bytes = body::to_bytes(resp.into_body()).await.unwrap();
    if !body_bytes.is_empty() {
        // no change
        let info: serde_json::Value = serde_json::from_slice(&body_bytes.to_vec()).unwrap();
        println!("value:{:?}", info);
    }

    Ok(())
}