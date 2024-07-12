use proxy_load_balancer::{domain::ServerType, services::{LoadBalancer, RoundRobinStrategy}, utils::init_tracing, Application};
use tokio::sync::RwLock;
use std::sync::Arc;

#[tokio::main]
async fn main () {
    init_tracing().expect("Tracing failed to start");
    let address = "127.0.0.1:4000";

    let worker_hosts = vec![
        ServerType::new("Server 0".to_string(), "http://localhost:3000".to_string()),
        ServerType::new("Server 1".to_string(), "http://localhost:3001".to_string()),
        ServerType::new("Server 2".to_string(), "http://localhost:3002".to_string()),
        ServerType::new("Server 3".to_string(), "http://localhost:3003".to_string()),
    ];

    let strategy = Arc::new(RwLock::new(RoundRobinStrategy::new()));
    let app_load_balancer = Arc::new(RwLock::new(LoadBalancer::new(worker_hosts, strategy)));

    let app = Application::build(address, app_load_balancer).await.expect("Failed to build load balancer");

    app.run().await.expect("Failed to run Load balancer");
}
