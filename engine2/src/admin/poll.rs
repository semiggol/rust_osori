use tokio::{task, time};
use std::time::{Duration, UNIX_EPOCH};
use hyper::client::HttpConnector;
use serde_json::Value;
use serde_json::json;
use hyper::{Client, Request, Method, Body, StatusCode, body};
use std::time::SystemTime;

pub fn poll_to_admin(client: Client<HttpConnector>, v: Value){
    task::spawn( async move {
        let mut interval = time::interval(Duration::from_secs(5));
        let id = v["id"].as_str().unwrap();
        loop {
            let client2 = client.clone();
            interval.tick().await;
            let msg = make_poll_msg(id);
            send_poll_msg(msg, client2).await.unwrap();
        }
    });
}

fn make_poll_msg(id: &str) -> Request<Body>{
    let sys_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(e) => {
            panic!("error occurred while getting current time..");
        }
    };

    let body = json!({
	  "id": id,
      "time": sys_time,
      "totalMemory": 16247584,
      "usedMemory": 10032588,
      "usedCpu": 1.37,
      "usedNetworkTrafficIn": 10,
      "usedNetworkTrafficOut": 5,
      "clientCount": 0,
      "requestCount": 0,
      "responseCount": 0,
      "responseTime": 0,
      "responseStatus": [0, 0, 0, 0, 0],
      "activeRequests": [] ,
      "errorMessage": ""
    });

    Request::builder()
        .method(Method::POST)
        .uri("http://118.67.135.216:5581/poll")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string())).unwrap()
}

async fn send_poll_msg(req: Request<Body>, client: Client<HttpConnector>) -> Result<(), String> {
    let resp = match client.request(req).await {
        Ok(resp)  => resp,
        Err(e) => {
            return Err(e.message().to_string())
        }
    };

    let body_bytes = body::to_bytes(resp.into_body()).await.unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
/*
    let body_bytes = body::to_bytes(resp.into_body()).await.unwrap();
    let info: serde_json::Value = serde_json::from_slice(&body_bytes.to_vec()).unwrap();
    println!("value:{:?}", info);


    if resp.status() != StatusCode::OK {
        return Err(format!("Not 200 OK(status code:{}", resp.status()));
    }

 */
    Ok(())
}