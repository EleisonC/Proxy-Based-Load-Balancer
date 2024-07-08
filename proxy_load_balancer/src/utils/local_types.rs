use std::sync::Arc;
use crate::{domain::LoadBalancingStrategy, services::LoadBalancer};
use tokio::sync::RwLock;

pub type LoadBalancingStrategyType = Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;
pub type LoadBalancerType = Arc<RwLock<LoadBalancer>>;