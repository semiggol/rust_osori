use super::poll;
use crate::config::{api, args, system};
use crate::monitor;
use hyper::{body, Body, Client, Method, Request, StatusCode};
use monitor::system::{get_hostname, get_logical_cpus};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    id: String,
    engine_name: String,
    group_name: String,
    host_name: String,
    version: String,
    cpu: String,
    error_message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub api: Vec<api::DeserializedApi>,
    pub config: system::SystemConfig,
    pub id: String,
}

pub async fn handle(config: args::SystemConfig) -> Result<(), String> {
    let uri = format!("http://{}/register", config.admin_address);

    let message = make_register_message(config);

    // 1. connect and send register msg to admin
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(message))
        .unwrap();

    let client = Client::new();

    let resp = match client.request(req).await {
        Ok(resp) => resp,
        Err(e) => return Err(e.message().to_string()),
    };

    if resp.status() != StatusCode::OK {
        return Err(format!("Not 200 OK(status code:{})", resp.status()));
    }

    let body_bytes = body::to_bytes(resp.into_body()).await.unwrap();
    let info: RegisterResponse = serde_json::from_slice(&body_bytes.to_vec()).unwrap();

    
    // 2. process admin's register's response message
    let id = info.id.clone(); // todo: global variable1
    process_register_response_message(info);

    // 3. start to poll to admin every 5 seconds
    poll::handle(client, id);

    Ok(())
}

fn make_register_message(config: args::SystemConfig) -> String {
    let (host_name, cpus) = get_system_info();

    let engine_name = config.engine_name.unwrap_or_else(|| host_name.clone());
    let group_name = config.group_name.unwrap_or_default();

    let message = RegisterRequest {
        id: String::from(""),
        engine_name,
        group_name,
        host_name,
        version: String::from("2.1"),
        cpu: cpus.to_string(),
        error_message: String::from(""),
    };

    serde_json::to_string(&message).unwrap()
}

fn get_system_info() -> (String, usize) {
    use sysinfo::{System, SystemExt};

    // monitoring info
    let mut my_system = System::new_all();
    my_system.refresh_all();

    // hostname
    let hostname = get_hostname(&my_system);

    // logical cpu count
    let cpus = get_logical_cpus(&my_system);

    (hostname, cpus)
}

fn process_register_response_message(info: RegisterResponse) {
  
  // todo: global variable!
  
  // 1. info.id
  // 2. info.config
  
  // 3. info.api
  api::insert_apis_into_new_map(info.api);
}