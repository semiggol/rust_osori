use std::task::{Context, Poll};
use http::{Request, Response};
use hyper::client::ResponseFuture;
use tower_service::Service;
use crate::service::access_log::AccessLogRequestBody;
use crate::service::route::Route;
use crate::tls::tls_connector::make_http_or_https_client;

#[derive(Debug, Clone)]
pub struct ProxyService;

impl Service<Request<AccessLogRequestBody<hyper::Body>>> for ProxyService
{
    type Response = Response<hyper::Body>;
    type Error = hyper::Error;
    type Future = ResponseFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<AccessLogRequestBody<hyper::Body>>) -> Self::Future {
        let route = req.extensions_mut().remove::<Route>().expect("route not found.");
        let client = make_http_or_https_client();
        *req.uri_mut() = route.target_servers[0].parse().unwrap();
        client.request(req.map(|inner| inner.inner))
    }
}
