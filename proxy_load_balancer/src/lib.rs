use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub struct Application {
    addr: SocketAddr,
    listener: TcpListener,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let addr = address.parse()?;

        let listener = TcpListener::bind(addr).await?;

        let app_inst = Application {
            addr,
            listener
        };
        Ok(app_inst)
    }

    pub async fn run(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Listening on http://{}", self.addr);

        loop {
            let (stream, _) = self.listener.accept().await?;
            let io_stream = TokioIo::new(stream);
            let io = http1::Builder::new().serve_connection(io_stream, service_fn(hello));

            tokio::task::spawn(async move {
                if let Err(err) = io.await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn hello(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
