use errors::*;
use eth2_simulator::*;
use simulator::Simulator;

#[test]
fn process_slots_happy() {
    let mut simulator = Simulator::new();
    // Process until the end of the first slot of epoch 3.
    let end_slot = compute_start_slot_at_epoch(3);
    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());
    // Simulator
    // Next slot to be processed.
    assert_eq!(end_slot + 1, simulator.slot);

    // BeaconChain
    // Next slot to be processed.
    let beacon_chain = simulator.beacon_chain;
    assert_eq!(end_slot + 1, beacon_chain.slot);

    // Finality
    let finalized_epoch = compute_epoch_at_slot(end_slot) - 2;
    let finalized_slot = compute_start_slot_at_epoch(finalized_epoch);
    assert_eq!(finalized_epoch, beacon_chain.finalized_checkpoint.epoch);
    assert_eq!(
        beacon_chain.blocks[finalized_slot as usize].header().root(),
        beacon_chain.finalized_checkpoint.root
    );
    assert_eq!(
        finalized_slot,
        beacon_chain.get_finalized_blocks().last().unwrap().slot
    );
    assert_eq!(
        beacon_chain.finalized_checkpoint.root,
        beacon_chain
            .get_finalized_blocks()
            .last()
            .unwrap()
            .header()
            .root()
    );

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        // Beacon blocks are proposed at every slot.
        assert_eq!(
            processed_slot,
            beacon_chain.blocks[processed_slot as usize].slot
        );
        if processed_slot < end_slot - 1 {
            // Verify the hash chain.
            assert_eq!(
                beacon_chain.blocks[processed_slot as usize].header().root(),
                beacon_chain.blocks[processed_slot as usize + 1].parent_root
            );
        }

        // Shard header existence.
        let shard_set = (0..SHARD_NUM as Shard).collect::<HashSet<Shard>>();
        let mut proposed_shard_set: HashSet<Shard> = HashSet::new();
        for shard_header in beacon_chain.blocks[processed_slot as usize]
            .shard_headers
            .iter()
        {
            assert_eq!(processed_slot, shard_header.message.slot);
            proposed_shard_set.insert(shard_header.message.shard);
        }
        assert_eq!(shard_set, proposed_shard_set);

        // Shard header confirmation.
        for pending_shard_header in beacon_chain.states[processed_slot as usize]
            .current_epoch_pending_shard_headers
            .iter()
        {
            assert!(pending_shard_header.confirmed);
        }

        // Beacon state are proposed at every slot.
        assert_eq!(
            processed_slot,
            beacon_chain.states[processed_slot as usize].slot
        );

        // The grandparent epoch's checkpoint is always assumed to be finalized.
        // The next beacon block learns the finalized checkpoint in the state.
        if compute_epoch_at_slot(processed_slot) < 2 {
            assert_eq!(
                Checkpoint::genesis_finalized_checkpoint(),
                beacon_chain.states[processed_slot as usize + 1].finalized_checkpoint
            );
        } else if processed_slot < end_slot - 1 {
            let finalized_epoch = compute_epoch_at_slot(processed_slot) - 2;
            let finalized_slot = compute_start_slot_at_epoch(finalized_epoch);
            assert_eq!(
                finalized_epoch,
                beacon_chain.states[processed_slot as usize + 1]
                    .finalized_checkpoint
                    .epoch
            );
            assert_eq!(
                beacon_chain.blocks[finalized_slot as usize].header().root(),
                beacon_chain.states[processed_slot as usize + 1]
                    .finalized_checkpoint
                    .root
            );
        }
    }
}

// TODO: Add tests for `BeaconState`'s `previous_epoch_pending_shard_headers` and `grandparent_epoch_confirmed_commitments`.
#[test]
fn process_slots_with_bids() {
    let mut simulator = Simulator::new();
    // In epoch 0, 1, and 2, bids are published.
    let bid_end_slot = compute_start_slot_at_epoch(3);
    // In epoch 3, 4, and 5, no bid is published to the end.
    let end_slot = compute_start_slot_at_epoch(6);

    // The gas price at the start of the currrent epoch.
    let mut current_gasprice = simulator.beacon_chain.state.shard_gasprice;
    // The gasprice at the end of the current epoch.
    let mut child_epoch_gasprice = INIT_SHARD_GASPRICE;
    // The gasprice at the end of the next epoch.
    let mut grandchild_epoch_gasprice = INIT_SHARD_GASPRICE;

    // Epochs with bid.
    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let mut low_fee_bid_ids: HashSet<(Shard, Slot, DataCommitment)> = HashSet::new();
        let mut high_fee_bid_ids: HashSet<(Shard, Slot, DataCommitment)> = HashSet::new();
        if processed_slot <= bid_end_slot {
            for shard in 0..SHARD_NUM as Shard {
                // Publish a bid with low fee and high fee.
                let low_fee_bid = Bid {
                    shard,
                    slot: processed_slot,
                    commitment: DataCommitment::dummy_from_bytes(
                        &format!(
                            "Bid with a low fee: Slot {}, Shard {}",
                            processed_slot, shard
                        )
                        .into_bytes(),
                    ),
                    fee: 1,
                };
                let high_fee_bid = Bid {
                    shard,
                    slot: processed_slot,
                    commitment: DataCommitment::dummy_from_bytes(
                        &format!(
                            "Bid with a high fee: Slot {}, Shard {}",
                            processed_slot, shard
                        )
                        .into_bytes(),
                    ),
                    fee: 21000 * 100,
                };
                low_fee_bid_ids.insert((
                    low_fee_bid.shard,
                    low_fee_bid.slot,
                    low_fee_bid.commitment.clone(),
                ));
                high_fee_bid_ids.insert((
                    high_fee_bid.shard,
                    high_fee_bid.slot,
                    high_fee_bid.commitment.clone(),
                ));
                let result = simulator.publish_bid(low_fee_bid);
                assert!(result.is_ok());
                let result = simulator.publish_bid(high_fee_bid);
                assert!(result.is_ok());
            }
        }

        let result = simulator.process_slots_happy(processed_slot);
        assert!(result.is_ok());

        if processed_slot <= bid_end_slot {
            // Only the bid with the highest fee is included in the shard header.
            let proposed_bid_ids: HashSet<(Shard, Slot, DataCommitment)> = simulator
                .shards
                .iter()
                .map(|shard| {
                    let header = shard
                        .proposed_headers
                        .last()
                        .unwrap()
                        .clone()
                        .unwrap()
                        .message;
                    (header.shard, header.slot, header.commitment)
                })
                .collect();
            assert_eq!(high_fee_bid_ids, proposed_bid_ids);
            assert!(low_fee_bid_ids.is_disjoint(&proposed_bid_ids));

            // All the bid with the highest fee is included and confirmed in the beacon chain.
            let confirmed_bid_ids: HashSet<(Shard, Slot, DataCommitment)> = simulator
                .beacon_chain
                .states
                .last()
                .unwrap()
                .current_epoch_pending_shard_headers
                .iter()
                .filter_map(|header| {
                    if header.confirmed {
                        Some((header.shard, header.slot, header.commitment.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            assert!(high_fee_bid_ids.is_subset(&confirmed_bid_ids));

            // No bid with lower fee is included in the beacon chain.
            let included_bid_ids: HashSet<(Shard, Slot, DataCommitment)> = simulator
                .beacon_chain
                .states
                .last()
                .unwrap()
                .current_epoch_pending_shard_headers
                .iter()
                .map(|header| (header.shard, header.slot, header.commitment.clone()))
                .collect();
            assert!(low_fee_bid_ids.is_disjoint(&included_bid_ids));
        }

        // Verify shard gasprice.
        // The data whose commitments confirmed in the current epoch affects the gas price at the end of the next epoch.
        grandchild_epoch_gasprice = compute_updated_gasprice(
            grandchild_epoch_gasprice,
            // grandchild_epoch_gasprice = BeaconChain::compute_updated_gasprice(grandchild_epoch_gasprice,
            high_fee_bid_ids.iter().map(|bid_id| bid_id.2.length).sum(),
        );
        if (processed_slot + 1) % SLOTS_PER_EPOCH == 0 {
            // Shard gasprice is updated at the end of an epoch.
            assert_eq!(
                simulator.beacon_chain.state.shard_gasprice,
                child_epoch_gasprice
            );
            child_epoch_gasprice = grandchild_epoch_gasprice;
            current_gasprice = simulator.beacon_chain.state.shard_gasprice;
        } else {
            // Otherwise, shard gasprice is not updated.
            assert_eq!(
                simulator.beacon_chain.state.shard_gasprice,
                current_gasprice
            )
        }
    }
}

#[test]
fn process_slots_without_shard_data_inclusion() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(2);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let mut commitments: HashSet<DataCommitment> = HashSet::new();
        for shard in 0..SHARD_NUM as Shard {
            // Publish a bid.
            let bid = Bid {
                shard,
                slot: processed_slot,
                commitment: DataCommitment::dummy_from_bytes(
                    &format!("Slot {}, Shard {}", processed_slot, shard).into_bytes(),
                ),
                fee: 1,
            };
            commitments.insert(bid.commitment.clone());
            let result = simulator.publish_bid(bid);
            assert!(result.is_ok());
        }

        let result: Result<(), SlotProcessingError>;
        if processed_slot % 2 == 0 {
            result = simulator.process_slots_without_shard_data_inclusion(processed_slot);
            let included_commitments: HashSet<DataCommitment> = simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .iter()
                .map(|signed_header| signed_header.message.commitment.clone())
                .collect();
            assert_eq!(
                (0..SHARD_NUM)
                    .map(|_| DataCommitment::default())
                    .collect::<HashSet<DataCommitment>>(),
                included_commitments
            );
        } else {
            result = simulator.process_slots_happy(processed_slot);
            let included_commitments: HashSet<DataCommitment> = simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .iter()
                .map(|signed_header| signed_header.message.commitment.clone())
                .collect();
            assert_eq!(commitments, included_commitments);
        };
        assert!(result.is_ok());
    }
}

#[test]
fn publish_bids_without_shard_blob_proposal() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(2);

    for processed_slot in 0..end_slot {
        println!("Check the result of Slot {}", processed_slot);
        let mut commitments: HashSet<DataCommitment> = HashSet::new();
        for shard in 0..SHARD_NUM as Shard {
            // Publish a bid.
            let bid = Bid {
                shard,
                slot: processed_slot,
                commitment: DataCommitment::dummy_from_bytes(
                    &format!("Slot {}, Shard {}", processed_slot, shard).into_bytes(),
                ),
                fee: 1,
            };
            commitments.insert(bid.commitment.clone());
            let result = simulator.publish_bid(bid);
            assert!(result.is_ok());
        }

        let result: Result<(), SlotProcessingError>;
        if processed_slot % 2 == 0 {
            result = simulator.process_slots_without_shard_blob_proposal(processed_slot);
            for shard in simulator.shards.iter() {
                assert!(shard.proposed_headers.last().unwrap().is_none());
            }
            assert!(simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .is_empty());
        } else {
            result = simulator.process_slots_happy(processed_slot);
            for shard in simulator.shards.iter() {
                assert!(shard.proposed_headers.last().unwrap().is_some());
            }
            let included_commitments: HashSet<DataCommitment> = simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .iter()
                .map(|signed_header| signed_header.message.commitment.clone())
                .collect();
            assert_eq!(commitments, included_commitments);
        };
        assert!(result.is_ok());
    }
}

#[test]
fn grandparent_epoch_header_not_included() {
    let mut simulator = Simulator::new();
    // Note: `end_slot` is equal to the number of the slots to be processed.
    let end_slot = compute_start_slot_at_epoch(5);
    // Define the two consecutive "catastrophic" epoch where no shard header is included.
    let catastrophy_start_slot = compute_start_slot_at_epoch(3);
    let catastrophy_end_slot = compute_start_slot_at_epoch(5) - 1;
    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        if processed_slot < catastrophy_start_slot {
            result = simulator.process_slots_happy(processed_slot);
            assert!(simulator
                .beacon_chain
                .previous_epoch_shard_header_pool
                .is_empty());
            assert!(simulator
                .beacon_chain
                .current_epoch_shard_header_pool
                .is_empty());
            assert_eq!(
                SHARD_NUM as usize,
                simulator
                    .beacon_chain
                    .blocks
                    .last()
                    .unwrap()
                    .shard_headers
                    .len()
            );
        } else if processed_slot <= catastrophy_end_slot {
            // Catastrophic epochs without shard header inclusion.
            result = simulator.process_slots_without_shard_header_inclusion(processed_slot);
            assert!(simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .is_empty());
        } else {
            // Epochs after the catastrophy.
            result = simulator.process_slots_happy(processed_slot);
            assert!(
                simulator
                    .beacon_chain
                    .blocks
                    .last()
                    .unwrap()
                    .shard_headers
                    .len()
                    >= SHARD_NUM as usize
            );
        }
        assert!(result.is_ok());
        let processed_epoch = compute_epoch_at_slot(processed_slot);
        // Only the shard headers from the previous or current epoch can be included.
        for signed_header in simulator
            .beacon_chain
            .blocks
            .last()
            .unwrap()
            .shard_headers
            .iter()
        {
            assert!(
                (processed_epoch == compute_epoch_at_slot(signed_header.message.slot))
                    || (processed_epoch == compute_epoch_at_slot(signed_header.message.slot) + 1)
            );
        }
    }
    // Invariants about pending shard headers.
    for state in simulator.beacon_chain.states.iter() {
        for header in state.previous_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot) + 1
            );
        }
        for header in state.current_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot)
            );
        }
    }
}

#[test]
fn process_slots_without_shard_header_inclusion() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(2);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        if processed_slot % 2 == 0 {
            result = simulator.process_slots_without_shard_header_inclusion(processed_slot);
            assert!(simulator
                .beacon_chain
                .blocks
                .last()
                .unwrap()
                .shard_headers
                .is_empty());
        } else {
            result = simulator.process_slots_happy(processed_slot);
            assert_eq!(
                SHARD_NUM as usize * 2,
                simulator
                    .beacon_chain
                    .blocks
                    .last()
                    .unwrap()
                    .shard_headers
                    .len()
            );
        };
        assert!(result.is_ok());
    }
}

#[test]
fn process_slots_without_shard_header_confirmation() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(2);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        if processed_slot % 2 == 0 {
            result = simulator.process_slots_without_shard_header_confirmation(processed_slot);
            assert_eq!(
                SHARD_NUM as usize,
                simulator
                    .beacon_chain
                    .states
                    .last()
                    .unwrap()
                    .current_epoch_pending_shard_headers
                    .iter()
                    .filter(|header| !header.confirmed)
                    .count()
            );
        } else {
            result = simulator.process_slots_happy(processed_slot);
            assert_eq!(
                0,
                simulator
                    .beacon_chain
                    .states
                    .last()
                    .unwrap()
                    .current_epoch_pending_shard_headers
                    .iter()
                    .filter(|header| !header.confirmed)
                    .count()
            )
        };
        assert!(result.is_ok());
    }
}

#[test]
fn process_slots_without_beacon_chain_finality() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(15);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        let epoch = compute_epoch_at_slot(processed_slot);
        if epoch < 2 {
            continue;
        }
        if epoch % 3 == 0 {
            result = simulator.process_slots_without_beacon_chain_finality(processed_slot);
            assert_eq!(epoch - 3, simulator.beacon_chain.finalized_checkpoint.epoch);
        } else {
            result = simulator.process_slots_happy(processed_slot);
            assert_eq!(epoch - 2, simulator.beacon_chain.finalized_checkpoint.epoch);
        };
        assert!(result.is_ok());
    }
}

#[test]
fn process_slot_without_beacon_block_proposal() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(3);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        if processed_slot % 2 == 0 {
            result = simulator.process_slots_without_beacon_block_proposal(processed_slot);
            if processed_slot == GENESIS_SLOT {
                assert!(simulator.beacon_chain.blocks.is_empty());
                assert!(simulator.beacon_chain.states.is_empty());
            } else {
                assert_eq!(
                    processed_slot,
                    simulator.beacon_chain.blocks.last().unwrap().slot + 1
                );
                assert_eq!(
                    processed_slot,
                    simulator.beacon_chain.states.last().unwrap().slot + 1
                );
            }
        } else {
            result = simulator.process_slots_happy(processed_slot);
            assert_eq!(
                processed_slot,
                simulator.beacon_chain.blocks.last().unwrap().slot
            );
            assert_eq!(
                processed_slot,
                simulator.beacon_chain.states.last().unwrap().slot
            );
            assert_eq!(
                SHARD_NUM as usize * 2,
                simulator
                    .beacon_chain
                    .blocks
                    .last()
                    .unwrap()
                    .shard_headers
                    .len()
            );
        };
        assert_eq!(
            (processed_slot as usize + 1) / 2,
            simulator.beacon_chain.blocks.len()
        );
        assert_eq!(
            (processed_slot as usize + 1) / 2,
            simulator.beacon_chain.states.len()
        );
        assert!(result.is_ok());
    }
}

#[test]
fn recovery_from_epoch_without_beacon_block_proposal() {
    let mut simulator = Simulator::new();
    // Note: `end_slot` is equal to the number of the slots to be processed.
    let end_slot = compute_start_slot_at_epoch(5);
    // Define the "catastrophic" epoch where no beacon block is proposed.
    let catastrophic_epoch = 3;
    let catastrophy_start_slot = compute_start_slot_at_epoch(catastrophic_epoch);

    for processed_slot in 0..end_slot + 1 {
        println!("Check the result of Slot {}", processed_slot);
        let result: Result<(), SlotProcessingError>;
        if compute_epoch_at_slot(processed_slot) == catastrophic_epoch {
            // Catastrophic epoch without beacon block proposal
            result = simulator.process_slots_without_beacon_block_proposal(processed_slot);
            // The block proposed at the last slot before the catastrophy.
            assert_eq!(
                catastrophy_start_slot - 1,
                simulator.beacon_chain.blocks.last().unwrap().slot
            );
        } else if compute_epoch_at_slot(processed_slot) == catastrophic_epoch + 1 {
            result = simulator.process_slots_happy(processed_slot);
            // The number of processed slots after the catastrophy ends.
            let slot_in_epoch =
                (processed_slot - compute_start_slot_at_epoch(catastrophic_epoch + 1) + 1) as usize;
            // For each shard, at every slot, 4 (MAX_SHARD_HEADERS_PER_SHARD) headers are included and 1 new header is proposed.
            if slot_in_epoch <= 8 {
                // The previous epoch headers are fully included at the 8th slot in this epoch (bc SLOTS_PER_EPOCH = 32 = 8 * 4).
                assert_eq!(
                    (SLOTS_PER_EPOCH as usize - slot_in_epoch * 4) * SHARD_NUM as usize,
                    simulator
                        .beacon_chain
                        .previous_epoch_shard_header_pool
                        .len()
                );
                assert_eq!(
                    4 * SHARD_NUM as usize,
                    simulator
                        .beacon_chain
                        .blocks
                        .last()
                        .unwrap()
                        .shard_headers
                        .len()
                );
            } else if slot_in_epoch < 11 {
                // The number of headers of each shard left in the pool (initially SLOTS_PER_EPOCH) is reduced by 3 every slot.
                // The header pools get empty at the 11th slot in this epoch (bc SLOTS_PER_EPOCH = 32 = 10 * 3 + 2).
                assert_eq!(
                    (SLOTS_PER_EPOCH as usize - slot_in_epoch * 3) * SHARD_NUM as usize,
                    simulator.beacon_chain.current_epoch_shard_header_pool.len()
                );
                assert_eq!(
                    4 * SHARD_NUM as usize,
                    simulator
                        .beacon_chain
                        .blocks
                        .last()
                        .unwrap()
                        .shard_headers
                        .len()
                );
            } else if slot_in_epoch == 11 {
                assert!(simulator
                    .beacon_chain
                    .current_epoch_shard_header_pool
                    .is_empty());
                assert_eq!(
                    3 * SHARD_NUM as usize,
                    simulator
                        .beacon_chain
                        .blocks
                        .last()
                        .unwrap()
                        .shard_headers
                        .len()
                );
            }
        } else {
            result = simulator.process_slots_happy(processed_slot);
            assert!(simulator
                .beacon_chain
                .previous_epoch_shard_header_pool
                .is_empty());
            assert!(simulator
                .beacon_chain
                .current_epoch_shard_header_pool
                .is_empty());
            assert_eq!(
                SHARD_NUM as usize,
                simulator
                    .beacon_chain
                    .blocks
                    .last()
                    .unwrap()
                    .shard_headers
                    .len()
            );
        }
        assert!(result.is_ok());
    }
    let processed_epoch = compute_epoch_at_slot(end_slot);
    // A checkpoint must be defined for any epoch.
    assert_eq!(
        processed_epoch,
        simulator.beacon_chain.checkpoints.last().unwrap().epoch
    );
    assert_eq!(
        processed_epoch as usize + 1,
        simulator.beacon_chain.checkpoints.len()
    );
    for (epoch, checkpoint) in simulator.beacon_chain.checkpoints.iter().enumerate() {
        assert_eq!(epoch as Epoch, checkpoint.epoch);
    }

    // The block of the catastrophic epoch's checkpoint is the same with the next epochs' checkpoint.
    assert_eq!(
        simulator.beacon_chain.checkpoints[catastrophic_epoch as usize].root,
        simulator.beacon_chain.checkpoints[catastrophic_epoch as usize + 1].root
    );

    // Invariants about pending shard headers.
    for state in simulator.beacon_chain.states.iter() {
        for header in state.previous_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot) + 1
            );
        }
        for header in state.current_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot)
            );
        }
    }
}

#[test]
fn repeat_process_slots_random() {
    for _ in 0..32 {
        process_slots_random()
    }
}

fn process_slots_random() {
    let mut simulator = Simulator::new();
    let end_slot = compute_start_slot_at_epoch(8);
    let mut block_proposed_slots = 0;
    for processed_slot in 0..end_slot {
        println!("Check the result of Slot {}", processed_slot);
        // Start with slots without beacon blocks, and then process randomly.
        // Do not propose a block until the first slot of some epoch.
        let result = if processed_slot <= end_slot / 4 {
            simulator.process_slots_without_beacon_block_proposal(processed_slot)
        } else {
            simulator.process_slots_random(processed_slot)
        };
        assert!(result.is_ok());
        println!(
            "Beacon Params: {:?}",
            simulator.params.last().unwrap().beacon_params
        );
        // Note: Currently, all the shard uses the same params.
        println!(
            "Shard Params: {:?}",
            simulator
                .params
                .last()
                .unwrap()
                .shard_params
                .get(0)
                .unwrap()
        );
        // `simulator.slot` is the next slot to be processed.
        assert_eq!(processed_slot + 1, simulator.slot);
        // Verify the length of the main chain.
        if simulator
            .params
            .last()
            .unwrap()
            .beacon_params
            .beacon_block_proposed
        {
            block_proposed_slots += 1;
        }
        assert_eq!(block_proposed_slots, simulator.beacon_chain.blocks.len());
        assert_eq!(block_proposed_slots, simulator.beacon_chain.states.len());

        // Verify the hash chain.
        match block_proposed_slots {
            1 => assert_eq!(
                GENESIS_PARENT_ROOT,
                simulator.beacon_chain.blocks[0].parent_root
            ),
            s if s > 1 => assert_eq!(
                simulator.beacon_chain.blocks[block_proposed_slots - 2]
                    .header()
                    .root(),
                simulator.beacon_chain.blocks[block_proposed_slots - 1].parent_root
            ),
            _ => (),
        }
    }
    // End with happy slot to fill the checkpoint.
    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());

    let processed_epoch = compute_epoch_at_slot(end_slot);
    // A checkpoint must be defined for any epoch.
    assert_eq!(
        processed_epoch,
        simulator.beacon_chain.checkpoints.last().unwrap().epoch
    );
    assert_eq!(
        processed_epoch as usize + 1,
        simulator.beacon_chain.checkpoints.len()
    );

    // Invariants about pending shard headers.
    for state in simulator.beacon_chain.states.iter() {
        for header in state.previous_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot) + 1
            );
        }
        for header in state.current_epoch_pending_shard_headers.iter() {
            assert_eq!(
                compute_epoch_at_slot(state.slot),
                compute_epoch_at_slot(header.slot)
            );
        }
    }
}

#[test]
fn process_slot_validation() {
    let mut simulator = Simulator::new();
    let end_slot = 50;
    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());

    // Request past slot processing.
    let result = simulator.process_slots_happy(GENESIS_SLOT);
    assert_eq!(
        result,
        Err(SlotProcessingError::PastSlot {
            next: end_slot + 1,
            found: GENESIS_SLOT
        })
    );
    let result = simulator.process_slots_happy(end_slot);
    assert_eq!(
        result,
        Err(SlotProcessingError::PastSlot {
            next: end_slot + 1,
            found: end_slot
        })
    );
}

#[test]
fn publish_bid_validation() {
    let mut simulator = Simulator::new();
    let end_slot = 50;
    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());

    let good_bid = Bid {
        shard: 0,
        slot: end_slot + 1,
        commitment: DataCommitment::dummy_from_bytes(&String::from("Test bid").into_bytes()),
        fee: 1,
    };
    let result = simulator.publish_bid(good_bid.clone());
    assert!(result.is_ok());

    let mut old_bid1 = good_bid.clone();
    old_bid1.slot = GENESIS_SLOT;
    let result = simulator.publish_bid(old_bid1);
    assert_eq!(
        result,
        Err(BidPublicationError::PastSlot {
            next: end_slot + 1,
            found: GENESIS_SLOT
        })
    );

    let mut old_bid2 = good_bid.clone();
    old_bid2.slot = end_slot;
    let result = simulator.publish_bid(old_bid2);
    assert_eq!(
        result,
        Err(BidPublicationError::PastSlot {
            next: end_slot + 1,
            found: end_slot
        })
    );

    let mut large_bid = good_bid;
    large_bid.commitment.length = MAX_POINTS_PER_BLOCK + 1;
    let result = simulator.publish_bid(large_bid);
    assert_eq!(
        result,
        Err(BidPublicationError::TooLargeData {
            found: MAX_POINTS_PER_BLOCK + 1
        })
    );
}
