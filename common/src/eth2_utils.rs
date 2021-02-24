//! Utility functions in the Eth2 system
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::eth2_types::*;
use crate::eth2_config::{
    SLOTS_PER_EPOCH,
    TARGET_SAMPLES_PER_BLOCK,
    GASPRICE_ADJUSTMENT_QUOTIENT,
    MAX_GASPRICE,
    MIN_GASPRICE
};

/// Compute the epoch number at `slot`.
pub fn compute_epoch_at_slot(slot: Slot) -> Epoch {
    slot / SLOTS_PER_EPOCH as Epoch
}

/// Compute the start slot of `epoch`.
pub fn compute_start_slot_at_epoch(epoch: Epoch) -> Slot {
    epoch * SLOTS_PER_EPOCH as Epoch
}

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

// Calculate u64 hash.
// Ref: https://doc.rust-lang.org/std/hash/index.html#examples
pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Calculate dummy 32 bytes hash root.
/// TODO: Replace this with SSZ root.
pub fn root<T: Hash>(t: &T) -> Root {
    let mut hash: u64 = calculate_hash(t);
    let mut root: Vec<u8> = Vec::new();
    for _ in 0..4 {
        hash = calculate_hash(&hash);
        root.extend_from_slice(&u64::to_le_bytes(hash));
    }
    assert_eq!(32, root.len());
    H256::from_slice(&root)
}
