use http::{HeaderValue, Request, Response};
use std::{
    task::{Context, Poll},
};
use std::future::Future;
use std::pin::Pin;
use tower_layer::Layer;
use tower_service::Service;
use pin_project::pin_project;
use futures_util::ready;
use http::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN};
use crate::service::access_log::AccessLogResponseBody;

#[derive(Debug, Clone)]
pub struct CorsLayer;

impl<S> Layer<S> for CorsLayer {
    type Service = CorsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct CorsService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorsService<S>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
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
        let mut response = ready!(this.inner.poll(cx)?);
        response.headers_mut().insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        response.headers_mut().insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
        response.headers_mut().insert(ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("*"));

        Poll::Ready(Ok(response))
    }
}