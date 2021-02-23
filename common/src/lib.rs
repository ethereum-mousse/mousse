//! For the Eth2 part, based the following specs in the draft PR #2172
//! https://github.com/ethereum/eth2.0-specs/blob/074c09c018e77db8a8c88d9fa88f097fd646d5eb/specs/phase1/beacon-chain.md
//! https://github.com/ethereum/eth2.0-specs/blob/074c09c018e77db8a8c88d9fa88f097fd646d5eb/specs/phase1/data-availability-sampling.md#custom-types
pub mod eth2_types;
pub mod eth2_config;
pub mod bid;
#[macro_use]
extern crate serde_big_array;

#[cfg(test)]
mod tests {
    use super::*;
    use eth2_types::*;
    use eth2_config::*;


    #[test]
    fn calc_root() {
        let signed_headers: Vec<SignedShardHeader> = (0..SHARD_NUM * 2).map(|num|{
            SignedShardHeader::dummy_from_header(
                ShardHeader {
                    slot: (num / SHARD_NUM) as Slot,
                    shard: (num % SHARD_NUM) as Shard,
                    commitment: generate_dummy_from_string(&String::from(format!("Slot {}, Shard {}", num / SHARD_NUM, num % SHARD_NUM))),
                })}).collect();
        let state1 = BeaconState {
            slot: 0,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(
                signed_headers[..SHARD_NUM].iter().map(
                    |signed_header| PendingShardHeader::from_signed_shard_header(signed_header)
                ).collect::<Vec<PendingShardHeader>>()
            ),
        };
        let block1 = BeaconBlock {
            slot: 0,
            parent_root: H256::zero(),
            state_root: state1.root(),
            shard_headers: VariableList::from(signed_headers[..SHARD_NUM].to_vec()),
        };
        let state2 = BeaconState {
            slot: 1,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(
                signed_headers[SHARD_NUM..].iter().map(
                    |signed_header| PendingShardHeader::from_signed_shard_header(signed_header)
                ).collect::<Vec<PendingShardHeader>>()
            ),
        };
        let block2 = BeaconBlock {
            slot: 1,
            parent_root: block1.header().root(),
            state_root: state2.root(),
            shard_headers: VariableList::from(signed_headers[SHARD_NUM..].to_vec()),
        };
        println!("block1: {}", block1.header().root());
        println!("block2: {}", block2.header().root());
        println!("state1: {}", state1.root());
        println!("state2: {}", state2.root());
        assert_ne!(block1.header().root(), block2.header().root());
        assert_ne!(state1.root(), state2.root());
        assert_eq!(block1.header().root(), block2.parent_root);

        let another_state2 = BeaconState {
            slot: 1,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(
                signed_headers[SHARD_NUM..].iter().map(
                    |signed_header| PendingShardHeader::from_signed_shard_header(signed_header)
                ).collect::<Vec<PendingShardHeader>>()
            ),
        };
        let another_block2 = BeaconBlock {
            slot: 1,
            parent_root: block1.header().root(),
            state_root: state2.root(),
            shard_headers: VariableList::from(signed_headers[SHARD_NUM..].to_vec()),
        };
        assert_eq!(state2.root(), another_state2.root());
        assert_eq!(block2.header().root(), another_block2.header().root());

    }


    #[test]
    fn dummy_from_bytes() {
        check_dummy_from_string(String::from(""));
        check_dummy_from_string(String::from("hello"));
        compare_dummy_from_string(String::from("sharding"), String::from("sharding"));
        compare_dummy_from_string(String::from(""), String::from("Ethereum"));
        compare_dummy_from_string(String::from("Eth1"), String::from("Eth2"));
    }

    fn check_dummy_from_string(s: String) {
        let bytes = s.clone().into_bytes();
        let commitment = DataCommitment::dummy_from_bytes(&bytes);
        assert_eq!(calculate_hash(&bytes), commitment.point);
        assert_eq!((s.len() as f64 / BYTES_PER_POINT as f64).ceil() as u64, commitment.length);
    }

    fn generate_dummy_from_string(s: &String) -> DataCommitment {
        let bytes = s.clone().into_bytes();
        return DataCommitment::dummy_from_bytes(&bytes)
    }

    fn compare_dummy_from_string(s1: String, s2: String) {
        let commitment1 = generate_dummy_from_string(&s1);
        let commitment2 = generate_dummy_from_string(&s2);
        if s1 == s2 {
            assert_eq!(commitment1, commitment2);
        } else {
            assert_ne!(commitment1, commitment2);
        }
    }
}
