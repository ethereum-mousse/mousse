use crate::*;
use beacon_chain::*;
pub use errors::*;
use simulation_params::*;

/// Eth2 simulator.
pub struct Simulator {
    // The slot to be processed.
    // Note: The last processed slot is `self.slot - 1`.
    pub slot: Slot,
    pub beacon_chain: BeaconChain,
    pub shards: Vec<shard::Shard>,
    // Settings of the simulation of each slot.
    pub params: Vec<SimulationParams>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            slot: GENESIS_SLOT,
            beacon_chain: BeaconChain::new(),
            shards: (0..SHARD_NUM)
                .map(|shard_id| shard::Shard::new(shard_id as ShardId))
                .collect(),
            params: Vec::new(),
        }
    }

    /// Process to the given slot in a happy case.
    pub fn process_slots_happy(&mut self, slot: Slot) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params.push(SimulationParams::happy());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No data gets included in any shard.
    pub fn process_slots_without_shard_data_inclusion(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params
                .push(SimulationParams::no_shard_data_inclusion());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No shard blob is proposed in any shard.
    pub fn process_slots_without_shard_blob_proposal(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params.push(SimulationParams::no_shard_blob_proposal());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No shard header is included in any shard.
    pub fn process_slots_without_shard_header_inclusion(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params
                .push(SimulationParams::no_shard_header_inclusion());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No shard header is confirmed in any shard.
    pub fn process_slots_without_shard_header_confirmation(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params
                .push(SimulationParams::no_shard_header_confirmation());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No checkpoint gets finalized.
    pub fn process_slots_without_beacon_chain_finality(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params
                .push(SimulationParams::no_beacon_chain_finality());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. No beacon block is proposed.
    pub fn process_slots_without_beacon_block_proposal(
        &mut self,
        slot: Slot,
    ) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params
                .push(SimulationParams::no_beacon_block_proposal());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot. Fails randomly.
    pub fn process_slots_random(&mut self, slot: Slot) -> Result<(), SlotProcessingError> {
        while self.params.len() <= slot as usize {
            self.params.push(SimulationParams::random());
        }
        self.process_slots(slot)
    }

    /// Process to the given slot.
    fn process_slots(&mut self, slot: Slot) -> Result<(), SlotProcessingError> {
        if self.slot > slot {
            return Err(SlotProcessingError::PastSlot {
                next: self.slot,
                found: slot,
            });
        }
        while self.slot <= slot {
            self.process_slot();
            // Move to the next slot.
            self.slot += 1;
        }
        assert_eq!(self.params.len(), self.slot as usize);
        Ok(())
    }

    /// Process of a slot.
    fn process_slot(&mut self) {
        let params = &self.params[self.slot as usize];
        for shard in self.shards.iter_mut() {
            shard.process_slot(&params.shard_params[shard.shard_id as usize]);
            // The new shard header is published on the global subnet.
            // Assumption: If a shard blob is proposed, its header is published on the global subnet.
            if shard.proposed_headers[self.slot as usize].is_some() {
                self.beacon_chain.publish_shard_header(
                    shard.proposed_headers[self.slot as usize].clone().unwrap(),
                );
            }
        }
        self.beacon_chain.process_slot(&params.beacon_params);
    }

    /// Submit a bid.
    pub fn publish_bid(&mut self, bid: Bid) -> Result<(), BidPublicationError> {
        if bid.commitment.length > MAX_POINTS_PER_BLOCK {
            return Err(BidPublicationError::TooLargeData {
                found: bid.commitment.length,
            });
        }
        if bid.slot < self.slot {
            return Err(BidPublicationError::PastSlot {
                next: self.slot,
                found: bid.slot,
            });
        }
        self.shards[bid.shard as usize].publish_bid(bid);
        Ok(())
    }
}

impl Default for simulator::Simulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_simulator() {
        let simulator = Simulator::new();
        // Simulator
        assert_eq!(GENESIS_SLOT, simulator.slot);
        assert_eq!(SHARD_NUM as usize, simulator.shards.len());
        // BeaconChain
        let beacon_chain = simulator.beacon_chain;
        assert_eq!(GENESIS_SLOT, beacon_chain.slot);
        assert_eq!(
            Checkpoint::genesis_finalized_checkpoint(),
            beacon_chain.finalized_checkpoint
        );
        assert!(beacon_chain.blocks.is_empty());
        assert!(beacon_chain.states.is_empty());
        assert!(beacon_chain.previous_epoch_shard_header_pool.is_empty());
        assert!(beacon_chain.current_epoch_shard_header_pool.is_empty());
    }
}
