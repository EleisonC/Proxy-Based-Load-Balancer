// #[macro_use(defer)] extern crate scopeguard;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::error::Error;
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
use tokio::sync::{Mutex, Notify};
use utils::{LoadBalancerType, NotifyType, SwitchFlagType, SwitchLockType};
use hyper::body::Bytes;


pub mod services;
pub mod domain;
pub mod utils;

pub struct Application {
    addr: SocketAddr,
    listener: TcpListener,
    app_load_balancer: LoadBalancerType,
    switch_lock: SwitchLockType,
    switch_flag: SwitchFlagType,
    notify: NotifyType,
}

impl Application {
    pub async fn build(address: &str, app_load_balancer: LoadBalancerType) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let addr = address.parse()?;

        let listener = TcpListener::bind(addr).await?;
        let switch_lock = Arc::new(Mutex::new(()));

        let switch_flag = Arc::new(AtomicBool::new(false));  // Initially set to false
        let notify = Arc::new(Notify::new());
        
        let app_inst = Application {
            addr,
            listener,
            app_load_balancer,
            switch_lock,
            switch_flag,
            notify
        };
        Ok(app_inst)
    }

    pub async fn run(self) -> Result<(), LoadBalancerError> {
        tracing::info!("Listening on http://{}", self.addr);

        loop {
            let (stream, _) = self.listener.accept().await.map_err(|err| LoadBalancerError::UnexpectedError(err.into()))?;
            let io_stream = TokioIo::new(stream);
            let load_balancer = self.app_load_balancer.clone();
            let switch_lock = self.switch_lock.clone();
            let switch_flag = self.switch_flag.clone();
            let notify = self.notify.clone();
            tokio::task::spawn(async move {
                let io = http1::Builder::new().serve_connection(io_stream, service_fn(move |req| {
                    forward_to_load_balancer(req, load_balancer.clone(), switch_lock.clone(), switch_flag.clone(), notify.clone())
                }));
                if let Err(err) = io.await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

#[tracing::instrument(name = "Forward to load balancer", skip_all, err(Debug))]
async fn forward_to_load_balancer(
    req: Request<Incoming>, 
    load_balancer: LoadBalancerType,
    switch_lock: SwitchLockType,  // this should be an Arc<Mutex<()>> not Arc<RwLock<()>> to prevent race conditions
    switch_flag: SwitchFlagType,
    notify: NotifyType,
)
    -> Result<Response<Full<Bytes>>, Box<dyn Error + Send + Sync>> {
    match req.uri().path() {
        "/switch-strategy" => {
            println!("Switch is happening");
            // let _switch_guard = switch_lock.lock().await;
             // Set the switch flag to true
            switch_flag.store(true, Ordering::SeqCst);

            notify.notify_waiters();
            
            load_balancer.write().await.monitor_and_switch().await;

            switch_flag.store(false, Ordering::SeqCst);

            notify.notify_waiters();

            Ok(Response::builder()
               .status(StatusCode::OK)
               .body("Strategy switched successfully".into())
               .expect("Failed to create response"))
        },
        _ => {

            let lb = 
                load_balancer.read().await.forward_request(req, switch_flag, notify).await;

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
    }
    
}
