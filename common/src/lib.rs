//! For the Eth2 part, based the following specs in the draft PR #2172
//! https://github.com/ethereum/eth2.0-specs/blob/074c09c018e77db8a8c88d9fa88f097fd646d5eb/specs/phase1/beacon-chain.md
//! https://github.com/ethereum/eth2.0-specs/blob/074c09c018e77db8a8c88d9fa88f097fd646d5eb/specs/phase1/data-availability-sampling.md#custom-types
pub mod eth2_types;
pub mod eth2_config;
pub mod eth2_utils;
pub mod bid;
#[macro_use]
extern crate serde_big_array;
