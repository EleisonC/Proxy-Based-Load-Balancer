use std::error::Error;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use utils::LoadBalancerType;


pub mod services;
pub mod domain;
pub mod utils;

pub struct Application {
    addr: SocketAddr,
    listener: TcpListener,
    app_load_balancer: LoadBalancerType
}

impl Application {
    pub async fn build(address: &str, app_load_balancer: LoadBalancerType) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let addr = address.parse()?;

        let listener = TcpListener::bind(addr).await?;
        
        let app_inst = Application {
            addr,
            listener,
            app_load_balancer
        };
        Ok(app_inst)
    }

    pub async fn run(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing::info!("Listening on http://{}", self.addr);

        loop {
            let (stream, _) = self.listener.accept().await?;
            let io_stream = TokioIo::new(stream);
            let load_balancer = self.app_load_balancer.clone();
            let io = http1::Builder::new().serve_connection(io_stream, service_fn(move |req| {
                forward_to_load_balancer(req, load_balancer.clone())
            }));

            tokio::task::spawn(async move {
                if let Err(err) = io.await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn forward_to_load_balancer(req: Request<Incoming>, load_balancer: LoadBalancerType,) -> Result<Response<Incoming>, Box<dyn Error + Send + Sync>> {
    let response = load_balancer.write().await.forward_request(req).await?;
    Ok(response)
}
