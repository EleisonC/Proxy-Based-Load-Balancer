use std::sync::{Arc, Mutex};
use crate::{domain::{LoadBalancingStrategy, ServerType}, services::LoadBalancer};
use tokio::sync::RwLock;

pub type LoadBalancingStrategyType = Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;
pub type LoadBalancerType = Arc<RwLock<LoadBalancer>>;
pub type WokerHostType = Arc<Mutex<ServerType>>;
