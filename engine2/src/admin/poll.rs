use std::fmt::format;
use std::io;
use tokio::{task, time};
use std::time::Duration;
use hyper::Client;
use hyper::client::HttpConnector;
use serde_json::Value;
use tokio::task::JoinHandle;

pub fn poll_to_admin(client: Client<HttpConnector>, v: Value){
  println!("id={}", v["id"]);
  task::spawn( async {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
      interval.tick().await;
      make_poll_msg();
      //   send_poll_msg(client);
    }
  });
}

fn make_poll_msg(/*id: &str*/)/* -> &str*/{
  let msg = r#"{
	"id": "",
    "errorCode": []
  }"#;

}
