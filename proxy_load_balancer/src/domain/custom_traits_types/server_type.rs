use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::WokerHostType;



pub struct ServerType {
    pub name: String,
    pub address_ip: String,
    pub active_connections: usize
}

impl ServerType {
    pub fn new(name: String, address_ip: String) -> WokerHostType {
        Arc::new(Mutex::new(ServerType {
            name,
            address_ip,
            active_connections: 0
        }))
    }

    pub fn add_connection(&mut self) {
        self.active_connections += 1;
    }

    pub fn remove_connection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }
}