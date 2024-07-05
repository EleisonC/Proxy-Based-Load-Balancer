use std::sync::Arc;
use crate::domain::LoadBalancingStrategy;
use tokio::sync::RwLock;

pub type LoadBalancingStrategyType = Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;