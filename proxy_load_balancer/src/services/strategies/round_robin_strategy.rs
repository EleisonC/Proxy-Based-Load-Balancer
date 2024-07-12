use crate::{domain::LoadBalancingStrategy, utils::WokerHostType};

pub struct RoundRobinStrategy {
    current_worker: usize
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        RoundRobinStrategy {
            current_worker: 0
        }
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    #[tracing::instrument(name = "Get Worker via Round Robin Strategy", skip_all )]
    fn get_worker(&mut self, worker_hosts: Vec<WokerHostType>) -> String {
        let worker = worker_hosts.get(self.current_worker).unwrap();
        self.current_worker = (self.current_worker + 1) % worker_hosts.len();

        let server_address = worker.lock().unwrap().address_ip.clone();

        server_address
    }
}


