use super::poll;
use crate::monitor;
use crate::config::systemConfig;
use monitor::system::{ get_hostname, get_logical_cpus };
use hyper::{Client, Body, Method, Request, StatusCode, body};
use serde_json::json;

pub async fn register_to_admin(config: systemConfig) -> Result<(), String>{
  // 1. connect and send register msg to admin
  let req = Request::builder()
      .method(Method::POST)
      .uri(format!("http://{}/register", config.admin_address))
      .header("content-type", "application/json")
      .body(Body::from(make_msg(config))).unwrap();

  let client = Client::new();

  let resp = match client.request(req).await {
    Ok(resp)  => resp,
    Err(e) => {
      return Err(e.message().to_string())
    }
  };

  if resp.status() != StatusCode::OK {
    return Err(format!("Not 200 OK(status code:{})", resp.status()));
  }

  let body_bytes = body::to_bytes(resp.into_body()).await.unwrap();
  let info: serde_json::Value = serde_json::from_slice(&body_bytes.to_vec()).unwrap();

  // 2. start to poll to admin every 5 seconds
  poll::poll_to_admin(client, info);

  Ok(())
}

fn get_system_info() -> (String, usize){
  use sysinfo::{ System, SystemExt };

  // monitoring info
  let mut my_system = System::new_all();
  my_system.refresh_all();

  // hostname
  let hostname = get_hostname(&my_system);

  // logical cpu count
  let cpus = get_logical_cpus(&my_system);

  (hostname, cpus)
}

fn make_msg(config: systemConfig) -> String{
  let (host_name, cpus) = get_system_info();

  let engine_name = config.engine_name.unwrap_or_else(||host_name.clone());
  let group_name = config.group_name.unwrap_or_default();

  let msg = json!({
	"id": "",
	"engineName": engine_name,
    "groupName": group_name,
    "hostName": host_name,
    "vsersion": "2.1",
    "cpu": cpus,
    "errorMessage": ""
  });

  msg.to_string()
}

