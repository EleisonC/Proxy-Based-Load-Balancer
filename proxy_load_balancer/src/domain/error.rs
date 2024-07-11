use color_eyre::eyre::Report;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum LoadBalancerError {
    #[error("Failed to make connection to this Address")]
    InvalidAddress,
    #[error("Found no workers")]
    NoAvailableWorkers,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report)
}