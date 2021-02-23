//! Configuration of the Eth2 system.
use crate::eth2_types::*;
pub const GENESIS_SLOT: Slot = 0;
pub const GENESIS_EPOCH: Epoch = 0;
pub const GENESIS_PARENT_ROOT: Root = H256::zero();
pub const SLOTS_PER_EPOCH: u64 = 32;
/// Note: For now, we assume the number of shard is static, so do not define `MAX_SHARDS`.
pub const SHARD_NUM: usize = 64;

/// Ref: https://github.com/ethereum/eth2.0-specs/blob/074c09c018e77db8a8c88d9fa88f097fd646d5eb/specs/phase1/beacon-chain.md
pub const BYTES_PER_POINT :usize = 31;
pub const POINTS_PER_SAMPLE :usize = 8; // = 2 ** 3, 31 * 8 = 248 bytes
pub const MAX_SAMPLES_PER_BLOCK	:usize = 2048; // = 2 ** 11, 248 * 2,048 = 507,904 bytes
pub const TARGET_SAMPLES_PER_BLOCK :usize = 1024; // = 2 ** 10, 248 * 1,024 = 253,952 bytes
pub const MAX_POINTS_PER_BLOCK :usize = POINTS_PER_SAMPLE * MAX_SAMPLES_PER_BLOCK;
pub const MAX_SHARD_HEADERS_PER_SHARD: usize = 4;
pub const MAX_SHARD_HEADERS: usize = SHARD_NUM * MAX_SHARD_HEADERS_PER_SHARD;
