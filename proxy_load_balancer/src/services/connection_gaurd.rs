// use tokio::sync::Mutex;

use crate::utils::WokerHostType;

pub struct ConnectionGuard<'a> {
    worker: &'a WokerHostType
}

impl<'a> ConnectionGuard<'a> {
    pub async fn new(worker: &'a WokerHostType) -> Self {
        {
            let mut worker_guard = worker.lock().await;
            worker_guard.add_connection();
        }
        ConnectionGuard { worker }
    }
}

impl<'a> Drop for ConnectionGuard<'a> {
    fn drop(&mut self) {
        let worker = self.worker.clone();
        tokio::spawn(async move {
            let mut worker_guard = worker.lock().await;
            worker_guard.remove_connection();
        });
    }
}
