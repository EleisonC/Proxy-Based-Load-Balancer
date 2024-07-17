use crate::utils::WokerHostType;

pub trait LoadBalancingStrategy {
    fn current_strategy(&self) -> &str;
    fn get_worker(&mut self, worker_hosts: Vec<WokerHostType>) -> WokerHostType;
}