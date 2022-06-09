use bytes::Buf;
use futures_util::ready;
use http::{HeaderMap, Request, Response};
use http_body::Body;
use pin_project::pin_project;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

#[derive(Debug, Clone, Copy)]
pub struct AccessLogLayer {}

impl AccessLogLayer {
    pub fn new() -> Self {
        AccessLogLayer {}
    }
}

impl<S> Layer<S> for AccessLogLayer {
    type Service = AccessLog<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AccessLog::new(inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AccessLog<S> {
    inner: S,
}

impl<S> AccessLog<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    pub fn layer() -> AccessLogLayer {
        AccessLogLayer::new()
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AccessLog<S>
where
    S: Service<Request<AccessLogRequestBody<ReqBody>>, Response = Response<ResBody>>,
{
    type Response = Response<AccessLogResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let request_size = Arc::new(AtomicI64::new(0));
        ResponseFuture {
            inner: self.inner.call(req.map(|inner| AccessLogRequestBody {
                inner,
                request_size,
            })),
            metric: Some(Metric {
                id: 1,
                response_size: 0,
            }),
        }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: F,
    metric: Option<Metric>,
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<AccessLogResponseBody<B>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut response = ready!(this.inner.poll(cx)?);
        let mut metric = this.metric.take().unwrap();
        Poll::Ready(Ok(
            response.map(|inner| AccessLogResponseBody { inner, metric })
        ))
    }
}

#[pin_project]
pub struct AccessLogRequestBody<B> {
    #[pin]
    pub inner: B,
    request_size: Arc<AtomicI64>,
}

impl<B> Body for AccessLogRequestBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        let value = ready!(this.inner.poll_data(cx));
        if let Some(Ok(chunk)) = &value {
            this.request_size
                .fetch_add(chunk.remaining() as i64, Ordering::Relaxed);
        }
        Poll::Ready(value)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

#[pin_project]
pub struct AccessLogResponseBody<B> {
    #[pin]
    inner: B,
    metric: Metric,
}

impl<B> Body for AccessLogResponseBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        let value = ready!(this.inner.poll_data(cx));
        if let Some(Ok(chunk)) = &value {
            this.metric.response_size += chunk.remaining() as i64;
        }
        Poll::Ready(value)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

struct Metric {
    id: i32,
    response_size: i64,
}

impl Drop for Metric {
    fn drop(&mut self) {
        println!("request finished {} {}", self.id, self.response_size);
    }
}
