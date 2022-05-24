use http::{Request, Response};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug, Clone, Copy)]
pub struct RouteLayer {
}

impl RouteLayer {
    pub fn new() -> Self {
        RouteLayer { }
    }
}

impl<S> Layer<S> for RouteLayer {
    type Service = Route<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Route::new(inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Route<S> {
    inner: S,
}

impl<S> Route<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    // define_inner_service_accessors!();

    pub fn layer() -> RouteLayer {
        RouteLayer::new()
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Route<S>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        println!("route layer");
        ResponseFuture {
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: F,
}

impl<F, B, E> Future for ResponseFuture<F>
    where
        F: Future<Output = Result<Response<B>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut response = futures_core::ready!(this.inner.poll(cx)?);
        Poll::Ready(Ok(response))
    }
}
