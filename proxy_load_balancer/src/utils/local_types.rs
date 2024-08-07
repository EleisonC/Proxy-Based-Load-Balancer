use std::sync::{Arc, Mutex as std_mutex};
use crate::{domain::{LoadBalancingStrategy, ServerType}, services::LoadBalancer};
use tokio::sync::{RwLock, Mutex};

pub type LoadBalancingStrategyType = Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;
pub type LoadBalancerType = Arc<RwLock<LoadBalancer>>;
pub type WokerHostType = Arc<Mutex<ServerType>>;
pub type SwitchLockType = Arc<Mutex<()>>;
