
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper::{Client};
use hyper::client::HttpConnector;

pub fn make_http_or_https_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();
    Client::builder().build::<_, hyper::Body>(https)
}