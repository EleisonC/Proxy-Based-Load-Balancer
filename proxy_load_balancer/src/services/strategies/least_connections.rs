use crate::{domain::LoadBalancingStrategy,
    utils::WokerHostType};

#[derive(Default)]
pub struct LeastConnectionsStrategy;

impl LoadBalancingStrategy for LeastConnectionsStrategy {
    fn get_worker(&mut self, worker_hosts: Vec<WokerHostType>) -> WokerHostType {
        // let worker = worker_hosts.iter()
        //     .min_by_key(|server| server.lock().unwrap().active_connections)
        //     .expect("should have a server");

        // worker.clone()

        let worker = futures::future::join_all(
            worker_hosts.iter().min_by_key(|worker| async {
                let worker_guard = worker.lock().await;
                worker_guard.active_connections
            })
        ).await;

        
    }
    fn current_strategy(&self) -> &str {
        "Least Connections Strategy"
    }
}