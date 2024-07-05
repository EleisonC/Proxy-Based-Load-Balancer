use proxy_load_balancer::{services::RoundRobinStrategy, Application};
use tokio::sync::RwLock;
use std::sync::Arc;

#[tokio::main]
async fn main () {
    let address = "127.0.0.1:4000";

    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
    ];

    let strategy = Arc::new(RwLock::new(RoundRobinStrategy::new()));

    let app = Application::build(address, worker_hosts, strategy).await.expect("Failed to build load balancer");

    app.run().await.expect("Failed to run Load balancer");
}
