use hyper::{Client, Body, Method, Request, StatusCode, body};
use super::poll;

pub async fn register_to_admin() -> Result<(), String>{
  // 1. connect and send register msg to admin
  let msg = r#"{
	"id": "",
	"engineName": "macos",
    "groupName": "defaultGroup",
    "hostName": "MyHost",
    "vsersion": "2.1",
    "cpu": 4,
    "errorMessage": ""
  }"#;

  let req = Request::builder()
      .method(Method::POST)
      .uri("http://118.67.135.216:5581/register")
      .header("content-type", "application/json")
      .body(Body::from(msg)).unwrap();

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