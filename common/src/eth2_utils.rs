//! Utility functions in the Eth2 system
use std::cmp;
use crate::eth2_types::*;
use crate::eth2_config::{
    TARGET_SAMPLES_PER_BLOCK,
    GASPRICE_ADJUSTMENT_QUOTIENT,
    MAX_GASPRICE,
    MIN_GASPRICE
};

/// Compute the updated gasprice.
pub fn compute_updated_gasprice(prev_gasprice: Gwei, shard_block_length: u64) -> Gwei {
    if shard_block_length > TARGET_SAMPLES_PER_BLOCK {
        let delta = cmp::max(1, prev_gasprice * (shard_block_length - TARGET_SAMPLES_PER_BLOCK)
            / TARGET_SAMPLES_PER_BLOCK / GASPRICE_ADJUSTMENT_QUOTIENT);
        return cmp::min(prev_gasprice + delta, MAX_GASPRICE)

    } else {
        let delta = cmp::max(1, prev_gasprice * (TARGET_SAMPLES_PER_BLOCK - shard_block_length)
            / TARGET_SAMPLES_PER_BLOCK / GASPRICE_ADJUSTMENT_QUOTIENT);
        return cmp::max(prev_gasprice, MIN_GASPRICE + delta) - delta        
    }
}
