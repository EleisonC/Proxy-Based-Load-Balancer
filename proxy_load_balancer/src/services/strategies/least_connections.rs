use crate::{domain::LoadBalancingStrategy,
    utils::WokerHostType};

#[derive(Default)]
pub struct LeastConnectionsStrategy;

#[async_trait::async_trait]
impl LoadBalancingStrategy for LeastConnectionsStrategy {
    async fn get_worker(&self, worker_hosts: Vec<WokerHostType>) -> WokerHostType {
        let mut min_connections = usize::MAX;
        let mut least_connected_worker: Option<WokerHostType> = None;

        for worker_host in worker_hosts.iter() {
            let worker_guard = worker_host.read().await;
            if worker_guard.active_connection_count() < min_connections {
                min_connections = worker_guard.active_connection_count();
                least_connected_worker = Some(worker_host.clone());
            }
        };
        least_connected_worker.expect("No worker available")
    }   
    fn current_strategy(&self) -> &str {
        "Least Connections Strategy"
    }
}