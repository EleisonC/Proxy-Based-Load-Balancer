use crate::utils::WokerHostType;

pub trait LoadBalancingStrategy {
    fn current_strategy(&self) -> &str;
    async fn get_worker(&mut self, worker_hosts: Vec<WokerHostType>) -> WokerHostType;
}