use eth2_simulator::simulator;
use thiserror::Error;
use warp;

#[derive(Debug)]
pub struct SlotProcessingError(pub simulator::SlotProcessingError);

impl warp::reject::Reject for SlotProcessingError {}

pub fn slot_processing_error(e: simulator::SlotProcessingError) -> warp::reject::Rejection {
    warp::reject::custom(SlotProcessingError(e))
}

#[derive(Debug)]
pub struct BidPublicationError(pub simulator::BidPublicationError);

impl warp::reject::Reject for BidPublicationError {}

pub fn bid_publication_error(e: simulator::BidPublicationError) -> warp::reject::Rejection {
    warp::reject::custom(BidPublicationError(e))
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failure rate must be a positive integer <= 1.0 (found {found:?})")]
    InvalidFailureRate { found: f32 },
}
#[derive(Debug)]
pub struct ConfigSetError(pub ConfigError);

impl warp::reject::Reject for ConfigSetError {}

pub fn config_set_error(e: ConfigError) -> warp::reject::Rejection {
    warp::reject::custom(ConfigSetError(e))
}
