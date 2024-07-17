#[macro_use(defer)] extern crate scopeguard;

use std::{error::Error, ops::Deref};
use std::net::SocketAddr;

// use color_eyre::eyre::Context;
use domain::LoadBalancerError;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use serde_json::json;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use utils::LoadBalancerType;
use hyper::body::Bytes;


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

    pub async fn run(self) -> Result<(), LoadBalancerError> {
        tracing::info!("Listening on http://{}", self.addr);

        loop {
            let (stream, _) = self.listener.accept().await.map_err(|err| LoadBalancerError::UnexpectedError(err.into()))?;
            let io_stream = TokioIo::new(stream);
            let load_balancer = self.app_load_balancer.clone();
            tokio::task::spawn(async move {
                let io = http1::Builder::new().serve_connection(io_stream, service_fn(move |req| {
                    forward_to_load_balancer(req, load_balancer.clone())
                }));
                if let Err(err) = io.await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

#[tracing::instrument(name = "Forward to load balancer", skip_all, err(Debug))]
async fn forward_to_load_balancer(req: Request<Incoming>, load_balancer: LoadBalancerType,) -> Result<Response<Full<Bytes>>, Box<dyn Error + Send + Sync>> {
    println!("We are not locking anymore");
    let lb = load_balancer.read().await.forward_request(req).await;
    // .read().await.forward_request(req).await?;
    // let checkstrategy = lb.strategy.write().await.current_strategy() == "Least Connections Strategy";
    // if !checkstrategy {
    //     lb.monitor_and_switch().await;
    // }
    let response = match lb {
        Ok(res) => Ok(res),
        Err(_) => {
            let error_message = json!({
                "error": "Internal Server Error",
            }).to_string();
            let error_body = Full::new(Bytes::from(error_message));
            // convert the error into an incoming
            // let error_message = Body::(error_message.to_string());
            let error_response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(error_body)
                .expect("Failed");

           Ok(error_response)
        }
    };
    response
}
