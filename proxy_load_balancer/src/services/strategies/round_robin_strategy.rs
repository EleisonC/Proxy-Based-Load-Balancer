use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::LoadBalancingStrategy, utils::WokerHostType};

pub struct RoundRobinStrategy {
    current_worker: AtomicUsize
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        RoundRobinStrategy {
            current_worker: AtomicUsize::new(0)
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancingStrategy for RoundRobinStrategy {
    #[tracing::instrument(name = "Get Worker via Round Robin Strategy", skip_all )]
    async fn get_worker(&self, worker_hosts: Vec<WokerHostType>) -> WokerHostType {
        let current_index = self.current_worker.fetch_add(1, Ordering::SeqCst) % worker_hosts.len();
        worker_hosts.get(current_index).unwrap().clone()
    }
    fn current_strategy(&self) -> &str {
        "Round Robin Strategy"
    }
}


