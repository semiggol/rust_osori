use super::poll;
use super::super::monitor;

use monitor::system::{ get_hostname, get_logical_cpus };
use hyper::{Client, Body, Method, Request, StatusCode, body};
use serde_json::json;

pub async fn register_to_admin() -> Result<(), String>{
  let (hostname, cpus) = get_system_info();

  // 1. connect and send register msg to admin
  let msg = json!({
	"id": "",
	"engineName": "",
    "groupName": "",
    "hostName": hostname,
    "vsersion": "2.1",
    "cpu": cpus,
    "errorMessage": ""
  });

  let req = Request::builder()
      .method(Method::POST)
      .uri("http://118.67.135.216:5581/register")
      .header("content-type", "application/json")
      .body(Body::from(msg.to_string())).unwrap();

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
