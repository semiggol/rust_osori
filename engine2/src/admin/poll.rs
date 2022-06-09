use crate::admin::register::RegisterResponse;
use crate::config::api;
use crate::{config, monitor};
use hyper::client::HttpConnector;
use hyper::{body, Body, Client, Method, Request, StatusCode};
use monitor::system::{get_cpu_usage, get_memory_usage, get_network_usage};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::time::{Duration, UNIX_EPOCH};
use tokio::{task, time};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PollRequest {
    id: String,
    time: u128,
    total_memory: u64,
    used_memory: u64,
    used_cpu: f32,
    used_network_traffic_in: u64,
    used_network_traffic_out: u64,
    client_count: usize,
    request_count: usize,
    response_count: usize,
    response_time: usize,
    response_status: Vec<usize>,
    active_requests: Vec<ActiveRequestInfo>,
    error_message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActiveRequestInfo {
    api_name: String,
    api_version: usize,
    elapsed_time: usize,
}

#[derive(Serialize, Deserialize)]
struct PollResponse {
    action: String,
    #[serde(default)]
    api: Vec<api::DeserializedApi>,
}

pub fn handle(client: Client<HttpConnector>, id: String) {
    task::spawn(async move {
        // interval
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            let message = make_poll_message(id.clone());
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
    use sysinfo::{System, SystemExt};

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
        cpu_usage,
    }
}

fn make_poll_message(id: String) -> String {
    // get the information needed to make a poll message
    let sys_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(e) => {
            panic!("error occurred while getting current time: {}", e);
        }
    };
    let monitoring_info = get_monitoring_info();

    let message = PollRequest {
        id: id,
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
        .body(Body::from(body))
        .unwrap();

    let resp = match client.request(req).await {
        Ok(resp) => resp,
        Err(e) => return Err(e.message().to_string()),
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
        api::insert_apis_into_new_map(info.api);
    } else if info.action.eq("config") {
        // ToDo: 없어짐 ^^
    } else if info.action.eq("shutdown") {
        // ToDo:
    } else if info.action.eq("restart") {
        // ToDo:
    }
}
