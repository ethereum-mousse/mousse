//! Custom types in the fee market of Eth2.
//! Ref: https://ethresear.ch/t/a-fee-market-contract-for-eth2-shards-in-eth1/8124

use crate::eth2_types::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Bid {
    pub shard: Shard,
    pub slot: Slot,
    pub commitment: DataCommitment,
    pub fee: Gwei,
}
