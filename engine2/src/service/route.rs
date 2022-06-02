use http::{Request, Response};
use std::{
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug, Clone)]
pub struct Route {
    pub target_servers: Vec<String>
}

pub fn dummy_route() -> Route {
    Route {
        target_servers: vec!["https://httpbin.org".into()]
    }
}

#[derive(Debug, Clone)]
pub struct RouteLayer {
    route: Route,
}

impl RouteLayer {
    pub fn new(route: Route) -> Self {
        RouteLayer { route }
    }
}

impl<S> Layer<S> for RouteLayer {
    type Service = RouteService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RouteService { inner, route: self.route.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct RouteService<S> {
    inner: S,
    route: Route,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RouteService<S>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        req.extensions_mut().insert(self.route.clone());
        println!("Route complete: {}", self.route.target_servers[0]);
        self.inner.call(req)
    }
}

