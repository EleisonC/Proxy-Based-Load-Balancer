use proxy_load_balancer::Application;

#[tokio::main]
async fn main () {
    let address = "127.0.0.1:4000";
    let app = Application::build(address).await.expect("Failed to build load balancer");

    app.run().await.expect("Failed to run Load balancer");
}
