use proxy_load_balancer::{domain::ServerType, services::{LeastConnectionsStrategy, LoadBalancer, RoundRobinStrategy}, utils::init_tracing, Application};
use tokio::sync::RwLock;
use std::sync::Arc;

#[tokio::main]
async fn main () {
    init_tracing().expect("Tracing failed to start");
    let address = "127.0.0.1:4000";

    let worker_hosts = vec![
        ServerType::new("Server 0".to_string(), "http://127.0.0.1:5000".to_string()),
        ServerType::new("Server 1".to_string(), "http://127.0.0.1:5001".to_string()),
        ServerType::new("Server 2".to_string(), "http://127.0.0.1:5002".to_string()),
        ServerType::new("Server 3".to_string(), "http://127.0.0.1:5003".to_string()),
        // ServerType::new("Server 4".to_string(), "http://127.0.0.1:5004".to_string()),
    ];

    let init_strategy = Arc::new(RwLock::new(LeastConnectionsStrategy::default()));
    let app_load_balancer = Arc::new(RwLock::new(LoadBalancer::new(worker_hosts, init_strategy)));

    let app = Application::build(address, app_load_balancer).await.expect("Failed to build load balancer");

    app.run().await.expect("Failed to run Load balancer");
}
