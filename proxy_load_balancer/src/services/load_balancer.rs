use std::{str::FromStr, sync::{atomic::Ordering, Arc}};
use tokio::{net::TcpStream, sync::RwLock};
use hyper::{body::Incoming,
    client::conn::http1::handshake, Request, Response, Uri};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use crate::{
    domain::{LoadBalancerError, LoadBalancingStrategy, StrategyType}, services::ConnectionGuard, 
    utils::{LoadBalancingStrategyType, NotifyType, SwitchFlagType, WokerHostType}};

use super::{LeastConnectionsStrategy, RoundRobinStrategy};

#[derive(Clone)]
pub struct LoadBalancer {
    worker_hosts: Vec<WokerHostType>,
    pub strategy: Arc<RwLock<StrategyType>>
}


impl LoadBalancer {
    pub fn new(worker_hosts: Vec<WokerHostType>, working_strategy: LoadBalancingStrategyType) -> Self {
        LoadBalancer {
            worker_hosts,
            strategy: Arc::new(RwLock::new(StrategyType::new(working_strategy))),
        }
    }

    #[tracing::instrument(name = "Forward Request", skip_all, err(Debug))]
    pub async fn forward_request(&self,
        req: Request<Incoming>,
        switch_flag: SwitchFlagType,
        notify: NotifyType
    ) -> Result<Response<Full<Bytes>>, LoadBalancerError> {

        while switch_flag.load(Ordering::SeqCst) {
            notify.notified().await;
        }
        
        let worker = { self.strategy.read().await.current_strategy.read().await.get_worker(self.worker_hosts.clone()).await.clone() };

        let (worker_name, mut worker_uri) = {
            let worker_guard = worker.read().await;
            (worker_guard.name.clone(), worker_guard.address_ip.clone())
        };

        tracing::info!("Forwarding request on {}", worker_name);
        
        let _guard = ConnectionGuard::new(&worker).await;
        
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

    #[tracing::instrument(name = "Monitor and switch strategy", skip_all)]
    pub async fn monitor_and_switch(&self) {
        // Clone the current strategy to avoid holding the lock while performing operations
        let strategy_clone = self.strategy.clone();
    
        let current_strategy = {
            let strategy_guard = strategy_clone.read().await;
            strategy_guard.get_current_strategy().await
        };
    
        let high_load = self.is_high_load().await;
        println!("Does my code even get here? What is my load -> {}", high_load);
    
        // Determine the new strategy
        let new_strategy: Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;
    
        if high_load && current_strategy == "Round Robin Strategy" {
            new_strategy = Arc::new(RwLock::new(LeastConnectionsStrategy::default()));
            tracing::info!("Switching to Least Connections Strategy");
        } else if !high_load && current_strategy == "Least Connections Strategy" {
            new_strategy = Arc::new(RwLock::new(RoundRobinStrategy::new()));
            tracing::info!("Switching to Round Robin Strategy");
        } else {
            return;
        }

        self.strategy.write().await.switch_strategy(new_strategy).await;
    }


    pub async fn is_high_load(&self) -> bool {
        let results = futures::future::join_all(
            self.worker_hosts.iter().map(|worker| async {
                let worker_guard = worker.read().await;
                println!("number of connections here {}", worker_guard.active_connection_count());
                worker_guard.active_connection_count() > 5
            })
        ).await;
        results.into_iter().any(|result| result)
    }
}
