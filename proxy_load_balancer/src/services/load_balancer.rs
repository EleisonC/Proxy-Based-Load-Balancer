use std::str::FromStr;
use tokio::net::TcpStream;
use hyper::{body::{Body, Incoming}, client::conn::http1::handshake, Request, Response, Uri};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use serde::Deserialize;
use hyper_util::rt::TokioIo;
use crate::{domain::{LoadBalancerError, ServerType}, utils::{LoadBalancingStrategyType, WokerHostType}};

#[derive(Clone)]
pub struct LoadBalancer {
    worker_hosts: Vec<WokerHostType>,
    strategy: LoadBalancingStrategyType
}


impl LoadBalancer {
    pub fn new(worker_hosts: Vec<WokerHostType>, strategy: LoadBalancingStrategyType) -> Self {
        LoadBalancer {
            worker_hosts,
            strategy
        }
    }

    #[tracing::instrument(name = "Forward Request", skip_all, err(Debug))]
    pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, LoadBalancerError> {
        let mut worker_uri = self.strategy.write().await.get_worker(self.worker_hosts.clone());

        tracing::info!("Forwarding request on {}", worker_uri);
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
        Ok(res_data)
    }
}
