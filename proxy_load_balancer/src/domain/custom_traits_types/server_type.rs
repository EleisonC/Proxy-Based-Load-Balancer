use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use tokio::sync::{Mutex, RwLock};
use crate::utils::WokerHostType;



pub struct ServerType {
    pub name: String,
    pub address_ip: String,
    pub active_connections: AtomicUsize
}

impl ServerType {
    pub fn new(name: String, address_ip: String) -> WokerHostType {
        Arc::new(RwLock::new(ServerType {
            name,
            address_ip,
            active_connections: AtomicUsize::new(0)
        }))
    }

    pub fn add_connection(&mut self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    pub fn remove_connection(&self) {
        let current_connections = self.active_connections.load(Ordering::SeqCst);
        if current_connections > 0 {
            self.active_connections.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub fn active_connection_count(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }
}