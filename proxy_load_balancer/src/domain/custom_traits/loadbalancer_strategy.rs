

pub trait LoadBalancingStrategy {
    fn get_worker(&mut self, worker_hosts: Vec<String>) -> String;
}