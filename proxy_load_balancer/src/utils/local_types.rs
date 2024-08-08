use std::sync::{atomic::AtomicBool, Arc, Mutex as std_mutex};
use crate::{domain::{LoadBalancingStrategy, ServerType}, services::LoadBalancer};
use tokio::sync::{Mutex, Notify, RwLock};

pub type LoadBalancingStrategyType = Arc<RwLock<dyn LoadBalancingStrategy + Send + Sync>>;
pub type LoadBalancerType = Arc<RwLock<LoadBalancer>>;
pub type WokerHostType = Arc<RwLock<ServerType>>;
pub type SwitchLockType = Arc<Mutex<()>>;
pub type SwitchFlagType = Arc<AtomicBool>;
pub type NotifyType = Arc<Notify>;
