use hyper::client::ResponseFuture;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error, Server};
use hyper_rustls::HttpsConnectorBuilder;
use hyper::{Body, Request, Response};
use futures_util::{TryFutureExt};

const API_SAMPLE_DOMAIN: &'static str = "https://httpbin.org";

fn find_api(req: &mut Request<Body>) {
    println!("osori> api_request({:?})", req.uri());

    for key in &["content-length", "transfer-encoding", "accept-encoding", "content-encoding"] {
        req.headers_mut().remove(*key);
    }

    let uri = req.uri();
    let uri_string = match uri.query() {
        None => format!("{}{}", API_SAMPLE_DOMAIN, uri.path()),
        Some(query) => format!("{}{}?{}", API_SAMPLE_DOMAIN, uri.path(), query),
    };
    // ToDo: remove unwrap
    *req.uri_mut() = uri_string.parse().unwrap();

    println!("osori> api_request -> new_request({})", uri_string);
}

fn modify_response_headers(mut res: Response<Body>) -> Result<Response<Body>, Error> {
    println!("osori> modify_response_headers({:?})", res);
    // add new header to response, ToDo(?): remove unwrap
    res.headers_mut().insert("JANG", "new_value_younghwi".parse().unwrap());

    Ok::<_, Error>(res)
}

#[tokio::main]
async fn main() {

    let in_addr = ([127, 0, 0, 1], 3000).into();

    // https client (hyper_rustls)
    let https = HttpsConnectorBuilder::new()
    .with_native_roots()
    .https_only()
    .enable_http1()
    .build();

    let client_main = Client::builder().build::<_, hyper::Body>(https);

    let make_service = make_service_fn(move |_conn| {
        let client = client_main.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                println!("osori> service_fn(req={:?})",  req.uri());

                find_api(&mut req);
                // ToDo: 실패시 loop {find_api()} 처리?
                // 참고로 해당 요청은 spawn 되어 처리 되겠지?
                let future: ResponseFuture = client.request(req);
                let future = future.and_then(|res| async move {
                    modify_response_headers(res)
                });
                future
            }))
        }
    });

    let server = Server::bind(&in_addr).serve(make_service);

    println!("Listening on http://{}", in_addr);
    println!("Proxying on {}", API_SAMPLE_DOMAIN);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}