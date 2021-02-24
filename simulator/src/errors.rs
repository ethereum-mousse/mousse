use crate::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SlotProcessingError {
    #[error("already processed slot (next slot is {next:?}, found {found:?})")]
    PastSlot {
        next: Slot,
        found: Slot,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum BidPublicationError {
    #[error("bid for already processed slot (next slot is {next:?}, found {found:?})")]
    PastSlot {
        next: Slot,
        found: Slot,
    },
    #[error("bid with too large data (max length is {}, found {found:?})", MAX_POINTS_PER_BLOCK)]
    TooLargeData {
        found: u64,
    },
}
