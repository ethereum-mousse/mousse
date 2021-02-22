use crate::*;
use simulation_params::BeaconSimulationParams;

/// Beacon chain consensus
pub struct BeaconChain {
    // The slot to be processed.
    // Note: The last processed slot is `self.slot - 1`.
    pub slot: Slot,
    // The latest finalized checkpoint.
    pub finalized_checkpoint: Checkpoint,
    // Beacon blocks in the main chain.
    // Note: There can be skipped slots.
    // Assumption: No reorg or equivocation. (At most one block exists for each slot.)
    pub blocks: Vec<BeaconBlock>,
    // Beacon state at block of the main chain.
    // Note: Should we define a state for a slot without beacon block proposal?
    pub states: Vec<BeaconState>,
    // Checkpoints of each epoch in the main chain.
    // Note: A block can be the checkpoint for multiple consecutive epochs.
    pub checkpoints: Vec<Checkpoint>,
    // Shard headers published but not included in the main chain.
    // The latter one in the list is the fresher.
    pub previous_epoch_shard_header_pool: Vec<SignedShardHeader>,
    pub current_epoch_shard_header_pool: Vec<SignedShardHeader>,
}

impl BeaconChain {
    pub fn new() -> Self {
        Self {
            slot: GENESIS_SLOT,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            blocks: Vec::new(),
            states: Vec::new(),
            checkpoints: Vec::new(),
            previous_epoch_shard_header_pool: Vec::new(),
            current_epoch_shard_header_pool: Vec::new(),
        }
    }

    /// Get finalized beacon blocks.
    pub fn get_finalized_blocks(&self) -> Vec<BeaconBlock> {
        // Return empty vector if no checkpoint is finalized.
        if self.finalized_checkpoint == Checkpoint::genesis_finalized_checkpoint() {
            Vec::new()
        } else {
            let latest_finalized_slot = compute_start_slot_at_epoch(self.finalized_checkpoint.epoch);
            let mut finalized_blocks: Vec<BeaconBlock> = Vec::new();
            for block in self.blocks.iter() {
                if block.slot > latest_finalized_slot {
                    break;
                }
                finalized_blocks.push(block.clone());
            }
            finalized_blocks
        }
    }

    /// Publish a shard header in the global subnet.
    pub fn publish_shard_header(&mut self, header: SignedShardHeader) {
        if compute_epoch_at_slot(header.message.slot) == compute_epoch_at_slot(self.slot) {
            self.current_epoch_shard_header_pool.push(header);
        } else if compute_epoch_at_slot(header.message.slot) + 1 == compute_epoch_at_slot(self.slot) {
            self.previous_epoch_shard_header_pool.push(header);
        }        
    }

    /// Process of a slot.
    /// Extend the main chain by a newly proposed block.
    pub fn process_slot(&mut self, params: &BeaconSimulationParams) {        
        if params.beacon_block_proposed {
            // Propose a new beacon block.
            self.append_new_block_to_chain(params.shard_headers_included, params.shard_headers_confirmed);
        }
        if params.beacon_chain_finalized {
            // Finalize a new checkpoint.
            self.progress_consensus();
        }
        if (self.slot + 1) % SLOTS_PER_EPOCH == 0 {
            // Reset the shard headers pool at the end of an epoch.
            self.previous_epoch_shard_header_pool = self.current_epoch_shard_header_pool.clone();
            self.current_epoch_shard_header_pool.clear();
        }
        self.slot += 1;
    }    

    /// Create new beacon state.
    /// This function is called before a block is proposed.
    fn create_new_state(&self, included_previous_epoch_shard_headers: &Vec<SignedShardHeader>, 
        included_current_epoch_shard_headers: &Vec<SignedShardHeader>, shard_headers_confirmed: bool) -> BeaconState {
        let mut new_previous_epoch_pending_shard_headers: Vec<PendingShardHeader> =
            included_previous_epoch_shard_headers.iter().map(|signed_header| PendingShardHeader::from_signed_shard_header(&signed_header)).collect();        
        let mut previous_epoch_pending_shard_headers: Vec<PendingShardHeader> = 
            if self.is_first_block_proposal() {
                Vec::new()
            } else if self.is_checkpoint_missing() {
                // Inherit from the previous epoch at a checkpoint. 
                VariableList::into(self.states.last().unwrap().current_epoch_pending_shard_headers.clone())
            } else {
                // Otherwise, inherit from the previous block in the current epoch.
                VariableList::into(self.states.last().unwrap().previous_epoch_pending_shard_headers.clone())
            };
        previous_epoch_pending_shard_headers.append(&mut new_previous_epoch_pending_shard_headers);

        let mut new_current_epoch_pending_shard_headers: Vec<PendingShardHeader> =
            included_current_epoch_shard_headers.iter().map(|signed_header| PendingShardHeader::from_signed_shard_header(&signed_header)).collect();        
        let mut current_epoch_pending_shard_headers: Vec<PendingShardHeader> = 
            if self.is_checkpoint_missing() {
                // Refresh at a checkpoint. 
                Vec::new()
            } else {
                // Otherwise, inherit from the previous block in the current epoch.
                VariableList::into(self.states.last().unwrap().current_epoch_pending_shard_headers.clone())
            };
        current_epoch_pending_shard_headers.append(&mut new_current_epoch_pending_shard_headers);

        if shard_headers_confirmed {
            for header in &mut previous_epoch_pending_shard_headers {
                header.confirmed = true;
            }
            for header in &mut current_epoch_pending_shard_headers {
                header.confirmed = true;
            }
        }

        return BeaconState {
            slot: self.slot,
            finalized_checkpoint: self.finalized_checkpoint.clone(),
            previous_epoch_pending_shard_headers: VariableList::from(previous_epoch_pending_shard_headers), 
            current_epoch_pending_shard_headers: VariableList::from(current_epoch_pending_shard_headers), 
        }
    }

    /// Create a new block and append to the main chain.
    fn append_new_block_to_chain(&mut self, shard_headers_included: bool, shard_headers_confirmed: bool) {
        // Shard headers to be included in the current slot's beacon block.
        // TODO: Curve this out to another method.
        let mut included_previous_epoch_shard_headers: Vec<SignedShardHeader> = Vec::new();
        let mut included_current_epoch_shard_headers: Vec<SignedShardHeader> = Vec::new();
        if shard_headers_included {
            // If the number of headers in the pool exeeds the limit, select from the older headers.
            if self.previous_epoch_shard_header_pool.len() > MAX_SHARD_HEADERS_PER_BLOCK {
                // included_previous_epoch_shard_headers = self.previous_epoch_shard_header_pool[..MAX_SHARD_HEADERS_PER_BLOCK].to_vec();
                // self.previous_epoch_shard_header_pool = self.previous_epoch_shard_header_pool[MAX_SHARD_HEADERS_PER_BLOCK..].to_vec();

                // Keep the fresher headers that are not selected in the pool.
                included_previous_epoch_shard_headers = self.previous_epoch_shard_header_pool.drain(..MAX_SHARD_HEADERS_PER_BLOCK).collect();
            } else {
                included_previous_epoch_shard_headers = self.previous_epoch_shard_header_pool.clone();
                let max_current_epoch_shard_headers = MAX_SHARD_HEADERS_PER_BLOCK - self.previous_epoch_shard_header_pool.len();
                self.previous_epoch_shard_header_pool.clear();
                if self.current_epoch_shard_header_pool.len() > max_current_epoch_shard_headers {
                    // included_current_epoch_shard_headers = self.current_epoch_shard_header_pool[..max_current_epoch_shard_headers].to_vec();
                    // self.current_epoch_shard_header_pool = self.current_epoch_shard_header_pool[max_current_epoch_shard_headers..].to_vec();

                    // Keep the fresher headers that are not selected in the pool.
                    included_current_epoch_shard_headers = self.current_epoch_shard_header_pool.drain(..max_current_epoch_shard_headers).collect();
                } else {
                    included_current_epoch_shard_headers = self.current_epoch_shard_header_pool.clone();
                    // All the published headers are included in the new beacon block.
                    self.current_epoch_shard_header_pool.clear();    
                }
            }
        }

        // Create the new beacon state.
        let state = self.create_new_state(&included_previous_epoch_shard_headers, &included_current_epoch_shard_headers, shard_headers_confirmed);
        self.states.push(state.clone());

        let parent_root = if self.is_first_block_proposal() {
            GENESIS_PARENT_ROOT
        } else {
            self.blocks.last().unwrap().header().root()
        };

        let mut included_shard_headers = included_previous_epoch_shard_headers;
        included_shard_headers.append(&mut included_current_epoch_shard_headers);
        let new_block = BeaconBlock {
            slot: self.slot,
            parent_root: parent_root,
            state_root: self.states.last().unwrap().root(),
            shard_headers: VariableList::from(included_shard_headers),
        };

        // Define checkpoints if necessary.
        // Ref: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/fork-choice.md#get_ancestor
        while self.is_checkpoint_missing() {
            self.checkpoints.push(Checkpoint {
                epoch: self.checkpoints.len() as Epoch, 
                root: new_block.header().root(),
            })
        }
        assert!(self.checkpoints.len() == compute_epoch_at_slot(self.slot) as usize + 1);

        self.blocks.push(new_block);
    }

    /// Progress consensus.
    /// Assumption: Checkpoints can be finalized only in the grandchild epochs.
    fn progress_consensus(&mut self) {
        if compute_epoch_at_slot(self.slot) < 2 {
            return
        }
        let finalized_epoch = compute_epoch_at_slot(self.slot) - 2;
        if ((self.finalized_checkpoint == Checkpoint::genesis_finalized_checkpoint()) | (self.finalized_checkpoint.epoch < finalized_epoch)) &&
            (self.checkpoints.len() > finalized_epoch as usize) {
            self.finalized_checkpoint = self.checkpoints[finalized_epoch as usize].clone();
        }
    }

    fn is_first_block_proposal(&self) -> bool {
        // Whether or not it is the first time to propsoe a beacon block.
        self.blocks.is_empty()
    }

    // Whether or not checkpoints are not enough.
    fn is_checkpoint_missing(&self) -> bool {        
        self.checkpoints.len() < compute_epoch_at_slot(self.slot) as usize + 1
    }
}
