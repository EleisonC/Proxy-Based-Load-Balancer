use std::str::FromStr;
use tokio::net::TcpStream;
use hyper::{body::Incoming, client::conn::http1::handshake, Request, Response, Uri};
use hyper_util::rt::TokioIo;
use crate::utils::LoadBalancingStrategyType;

#[derive(Clone)]
pub struct LoadBalancer {
    worker_hosts: Vec<String>,
    strategy: LoadBalancingStrategyType
}


impl LoadBalancer {
    pub fn new(worker_hosts: Vec<String>, strategy: LoadBalancingStrategyType) -> Result<Self, String> {
        if worker_hosts.is_empty() {
            return Err("No worker hosts provided".into());
        }
        Ok(LoadBalancer {
            worker_hosts,
            strategy
        })
    }

    pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<Incoming>, Box<dyn std::error::Error + Send + Sync>> {
        let mut worker_uri = self.strategy.write().await.get_worker(self.worker_hosts.clone());


        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        let new_uri = Uri::from_str(&worker_uri).unwrap();

        let host = new_uri.host().expect("new uri has no host");
        let port = new_uri.port_u16().unwrap();

        let address = format!("{}:{}", host, port);

        let stream = TcpStream::connect(address).await?;

        let io = TokioIo::new(stream);

        let (mut sender, conn) = handshake(io).await?;

        tokio::task::spawn( async move {
            if let Err(err) = conn.await {
                eprintln!("Error serving connection: {:?}", err);
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

        let res = sender.send_request(new_req).await?;

        let res_headers = res.headers().clone();

        let mut res_data = Response::builder()
            .status(res.status())
            .version(res.version())
            .body(res.into_body())
            .expect("response builder");

        for (key, value) in res_headers.iter() {
            res_data.headers_mut().insert(key, value.clone());
        }
        println!("\n\nDone!");

        Ok(res_data)
    }
}
