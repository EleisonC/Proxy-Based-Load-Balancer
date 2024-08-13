use crate::utils::{LoadBalancingStrategyType, WokerHostType};

#[async_trait::async_trait]
pub trait LoadBalancingStrategy {
    fn current_strategy(&self) -> &str;
    async fn get_worker(&self, worker_hosts: Vec<WokerHostType>) -> WokerHostType;
}

pub struct StrategyType {
    pub current_strategy: LoadBalancingStrategyType,
}

impl StrategyType {
    pub fn new(new_strategy: LoadBalancingStrategyType) -> Self {
        StrategyType { current_strategy: new_strategy }
    }

    pub async fn get_current_strategy(&self) -> String {
        self.current_strategy.read().await.current_strategy().to_owned()
    }

    pub async fn switch_strategy(&mut self, new_strategy: LoadBalancingStrategyType) {
        self.current_strategy = new_strategy;
        tracing::info!("Switched to {} strategy", self.current_strategy.read().await.current_strategy());
    }
}