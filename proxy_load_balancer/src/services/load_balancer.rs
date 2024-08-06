use std::{str::FromStr, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};
use hyper::{body::Incoming,
    client::conn::http1::handshake, Request, Response, Uri};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use crate::{domain::LoadBalancerError,
    utils::{LoadBalancingStrategyType, WokerHostType}};

use super::{LeastConnectionsStrategy, RoundRobinStrategy};

#[derive(Clone)]
pub struct LoadBalancer {
    worker_hosts: Vec<WokerHostType>,
    pub strategy: LoadBalancingStrategyType,
}


impl LoadBalancer {
    pub fn new(worker_hosts: Vec<WokerHostType>, strategy: LoadBalancingStrategyType) -> Self {
        LoadBalancer {
            worker_hosts,
            strategy,
        }
    }

    #[tracing::instrument(name = "Forward Request", skip_all, err(Debug))]
    pub async fn forward_request(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, LoadBalancerError> {
        let worker = { self.strategy.write().await.get_worker(self.worker_hosts.clone()).clone() };

        let worker_name = { worker.lock().unwrap().name.clone() };
        let mut worker_uri = { worker.lock().unwrap().address_ip.clone() };

        tracing::info!("Forwarding request on {}", worker_name);
        {
            worker.lock().unwrap().add_connection() 
        }
        defer! {
            worker.lock().unwrap().remove_connection()
        }
        
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        let new_uri = Uri::from_str(&worker_uri).unwrap();

        let host = new_uri.host().expect("new uri has no host");
        let port = new_uri.port_u16().unwrap();

        let address = format!("{}:{}", host, port);

        let stream = TcpStream::connect(address).await.map_err(|err|
            LoadBalancerError::UnexpectedError(err.into())
        )?;

        let io = TokioIo::new(stream);

        let (mut sender, conn) = handshake(io).await.map_err(|err|
            LoadBalancerError::UnexpectedError(err.into())
        )?;

        tokio::task::spawn( async move {
            if let Err(err) = conn.await {
                eprintln!("Error forwarding connection: {:?}", err);
            }
        });

        let authority = new_uri.authority().unwrap().clone();

        let headers = req.headers().clone();

        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .header(hyper::header::HOST, authority.as_str())
            .body(req.into_body())
            .expect("request builder");

        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        let res = sender.send_request(new_req).await.map_err(|err| LoadBalancerError::UnexpectedError(err.into()))?;

        let res_headers = res.headers().clone();

        let res_status = res.status();

        let res_version = res.version();

        // Create a Full<Bytes> from the collected bytes
        let body = res.collect().await.map_err(|err| LoadBalancerError::UnexpectedError(err.into()))?;

        let mut res_data = Response::builder()
            .status(res_status)
            .version(res_version)
            .body(Full::new(body.to_bytes()))
            .expect("response builder");

        for (key, value) in res_headers.iter() {
            res_data.headers_mut().insert(key, value.clone());
        }
        
        tracing::info!("Successfully forwarded the request. Done!!");
        let number = { worker.lock().unwrap().active_connections };
        println!("{worker_name}***has**{number}***Connections***");
        Ok(res_data)
    }

    #[tracing::instrument(name = "Monitor and switch strategy", skip_all)]
    pub async fn monitor_and_switch(&mut self) {
        let current_strategy = {self.strategy.read().await.current_strategy().to_owned()};

        if self.is_high_load() && current_strategy == "Round Robin Strategy" {
                let least_strategy = Arc::new(RwLock::new(LeastConnectionsStrategy::default()));
                self.strategy = least_strategy;
                tracing::info!("Switching to Least Connections strategy");
        } else if current_strategy == "Least Connections Strategy" {
                let round_robin = Arc::new(RwLock::new(RoundRobinStrategy::new()));
                self.strategy = round_robin;
                tracing::info!("Switching to Round Robin Strategy");
        }
    }
    pub fn is_high_load(&self) -> bool {
        self.worker_hosts.iter().any(|worker| worker.lock().unwrap().active_connections < 1)
    }
}
