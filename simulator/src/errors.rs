use crate::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SlotProcessingError {
    #[error("already processed slot (next slot is {next:?}, found {found:?})")]
    PastSlot { next: Slot, found: Slot },
}

#[derive(Error, Debug, PartialEq)]
pub enum BidPublicationError {
    #[error("bid for already processed slot (next slot is {next:?}, found {found:?})")]
    PastSlot { next: Slot, found: Slot },
    #[error(
        "bid with too large data (max length is {}, found {found:?})",
        MAX_POINTS_PER_BLOCK
    )]
    TooLargeData { found: u64 },
    #[error("bid with invalid commitment (expect {expect:?}, found {found:?})")]
    InvalidCommitment {
        expect: DataCommitment,
        found: DataCommitment,
    },
    // Note: This is only used in http_api.
    #[error("bid with invalid shard (expect {expect:?}, found {found:?})")]
    InvalidShard { expect: Shard, found: Shard },
}
