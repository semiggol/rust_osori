use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error, Server};
use hyper_rustls::{HttpsConnectorBuilder};
use hyper::{Body, Request, Response};
use futures_util::{TryFutureExt};

use core::task::{Context, Poll};
use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};

use futures::join;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::vec::Vec;
use std::{env, fs, io, sync};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::ServerConfig;


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

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

#[tokio::main]
async fn main() {

    // http/https client (hyper_rustls)
    let https = HttpsConnectorBuilder::new()
    .with_native_roots()
    .https_or_http()
    .enable_http1()
    .build();

    // proxy_client for clone()
    let proxy_client = Client::builder().build::<_, hyper::Body>(https);
    let client1 = proxy_client.clone();
    let client2 = proxy_client.clone();

    // ip address for http service
    let http_addr = ([127, 0, 0, 1], 3000).into();

    // make_service_fn/service_fn
    let http_service = make_service_fn(move |_conn| {
        println!("osori> http_service: make_service_fn(con={:?})",  _conn);
        let client1 = client1.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                println!("osori> service_fn(req={:?})",  req.uri());

                find_api(&mut req);
                // ToDo: 실패시 loop {find_api()} 처리?
                client1.request(req).and_then(|res| async move {
                    modify_response_headers(res)
                })
            }))
        }
    });

    // http server
    let http_server = Server::bind(&http_addr).serve(http_service);


    // First parameter is port number (optional, defaults to 1337)
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "3003".to_owned(),
    };
    let https_addr = format!("127.0.0.1:{}", port).parse().unwrap();

    // Build TLS configuration. ToDo: use configuration from admin
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs("ssl/sample.pem").unwrap();
        // Load private key.
        let key = load_private_key("ssl/sample.rsa").unwrap();
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| error(format!("{}", e))).unwrap();
        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
        //cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()]; // Todo: check http2
        cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
        sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let incoming = AddrIncoming::bind(&https_addr).unwrap();
    
    // make_service_fn/service_fn
    let https_service = make_service_fn(move |_conn| {
        println!("osori> https_service: make_service_fn(TLS client)");
        let client2 = client2.clone();

        async move {
            Ok::<_, Error>(service_fn(move |mut req| {
                println!("osori> service_fn(req={:?})",  req.uri());

                find_api(&mut req);
                // ToDo: 실패시 loop {find_api()} 처리?
                client2.request(req).and_then(|res| async move {
                    modify_response_headers(res)
                })
            }))
        }
    });

    // https server
    let https_server = Server::builder(TlsAcceptor::new(tls_cfg, incoming)).serve(https_service);

    println!("Listening on http://{}", http_addr);
    println!("Listening on https://{}", https_addr);
    println!("Proxying on {}", API_SAMPLE_DOMAIN);

    // Run the future, keep going until an error occurs.
    let (_http_result, _https_result) = join!(http_server, https_server);

}

/// --------------- hyper-rustls/examples/server.rs --------------- ///


enum State {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

// tokio_rustls::server::TlsStream doesn't expose constructor methods,
// so we have to TlsAcceptor::accept and handshake to have access to it
// TlsStream implements AsyncRead/AsyncWrite handshaking tokio_rustls::Accept first
pub struct TlsStream {
    state: State,
}

impl TlsStream {
    fn new(stream: AddrStream, config: Arc<ServerConfig>) -> TlsStream {
        let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
        TlsStream {
            state: State::Handshaking(accept),
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

pub struct TlsAcceptor {
    config: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl TlsAcceptor {
    pub fn new(config: Arc<ServerConfig>, incoming: AddrIncoming) -> TlsAcceptor {
        TlsAcceptor { config, incoming }
    }
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}


// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate".into()))?;
    Ok(certs
        .into_iter()
        .map(rustls::Certificate)
        .collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}