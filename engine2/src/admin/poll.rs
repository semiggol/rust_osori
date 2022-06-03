use crate::{monitor, config};
use crate::admin::register::{ RegisterResponse };
use tokio::{task, time};
use std::time::{Duration, UNIX_EPOCH};
use hyper::client::HttpConnector;
use hyper::{ Client, Request, Method, Body, body, StatusCode };
use std::time::SystemTime;
use monitor::system::{ get_memory_usage, get_network_usage, get_cpu_usage };
use serde::{ Serialize, Deserialize };
use crate::config::api;

#[derive(Serialize, Deserialize)]
struct PollRequest {
    id: String,
    time: u128,
    #[serde(rename = "totalMemory")]
    total_memory: u64,
    #[serde(rename = "usedMemory")]
    used_memory: u64,
    #[serde(rename = "usedCPU")]
    used_cpu: f32,
    #[serde(rename = "usedNetworkTrafficIn")]
    used_network_traffic_in: u64,
    #[serde(rename = "usedNetworkTrafficOut")]
    used_network_traffic_out: u64,
    #[serde(rename = "clientCount")]
    client_count: usize,
    #[serde(rename = "requestCount")]
    request_count: usize,
    #[serde(rename = "responseCount")]
    response_count: usize,
    #[serde(rename = "responseTime")]
    response_time: usize, 
    #[serde(rename = "responseStatus")]
    response_status: Vec<usize>,
    #[serde(rename = "activeRequests")]
    active_requests: Vec<ActiveRequestInfo>,
    #[serde(rename = "errorMessage")]
    error_message: String,
}

#[derive(Serialize, Deserialize)]
struct ActiveRequestInfo {
    #[serde(rename = "apiName")]
    api_name: String,
    #[serde(rename = "apiVersion")]
    api_version: usize,
    #[serde(rename = "elapsedTime")]
    elapsed_time: usize,
}

#[derive(Serialize, Deserialize)]
struct PollResponse {
    action: String,
    #[serde(default)]
    api: Vec<api::Api>,
}

pub fn handle(client: Client<HttpConnector>, info: RegisterResponse){
    task::spawn( async move {
        // get id
        let id = info.id.as_str();

        // process admin's api message: ToDo: move this code to register.rs
        manage_api_from_admin(info.api);

        // interval
        let mut interval = time::interval(Duration::from_secs(5));
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

    let message = PollRequest {
        id: id.to_string(),
        time: sys_time,
        total_memory: monitoring_info.memory_usage_total,
        used_memory: monitoring_info.memory_usage,
        used_cpu: monitoring_info.cpu_usage,
        used_network_traffic_in: monitoring_info.network_usage_in,
        used_network_traffic_out: monitoring_info.network_usage_out,
        client_count: 0,
        request_count: 0,
        response_count: 0,
        response_time: 0,
        response_status: vec![0, 0, 0, 0, 0],
        active_requests: vec![],
        error_message: String::from(""),
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
        let info: PollResponse = serde_json::from_slice(&body_bytes.to_vec()).unwrap();
        process_admin_message(info);
    }

    Ok(())
}

///! ToDo: add process by "action"
fn process_admin_message(info: PollResponse) {
    if info.action.eq("api") {
        manage_api_from_admin(info.api);
    }
    else if info.action.eq("config") {
        // ToDo: 없어짐 ^^
    }
    else if info.action.eq("shutdown") {
        // ToDo:
    }
    else if info.action.eq("restart") {
        // ToDo:
    }
}

fn manage_api_from_admin(apis: Vec<api::Api>) {
    api::bulk_insert_into_new_map(apis);
}