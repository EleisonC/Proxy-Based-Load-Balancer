use crate::utils::WokerHostType;

#[async_trait::async_trait]
pub trait LoadBalancingStrategy {
    fn current_strategy(&self) -> &str;
    async fn get_worker(&self, worker_hosts: Vec<WokerHostType>) -> WokerHostType;
}