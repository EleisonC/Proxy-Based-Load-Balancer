use crate::{domain::LoadBalancingStrategy,
    utils::WokerHostType};

#[derive(Default)]
pub struct LeastConnectionsStrategy;

impl LoadBalancingStrategy for LeastConnectionsStrategy {
    fn get_worker(&mut self, worker_hosts: Vec<WokerHostType>) -> WokerHostType {
        let worker = worker_hosts.iter()
            .min_by_key(|server| server.lock().unwrap().active_connections)
            .expect("should have a server");

        worker.clone()
    }
    fn current_strategy(&self) -> &str {
        "Least Connections Strategy"
    }
}