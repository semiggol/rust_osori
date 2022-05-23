use std::io;
use hyper::{Client, Body, Method, Request, Uri, StatusCode};

pub async fn register_to_admin() -> Result<(), &'static str>{
  println!("register_to_admin");
  let msg = r#"{
	  "id": "",
    "groupName": "TestGroup",
    "hostName": "MyHost",
    "vsersion": "2.1",
    "cpu": "4",
    "errorCode": []
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
      println!("{}",e.message().to_string());
      return Err("Fail to request to admin.")
    }
  };

  if resp.status() != StatusCode::OK {
    println!("resp: {}", resp.status());
    return Err("Not 200 OK");
  }

  println!("Response: {}", resp.status());

  Ok(())
}

