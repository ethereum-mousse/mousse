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
    pub states: Vec<BeaconState>,
    // Checkpoints of each epoch in the main chain.
    // Note: A block can be the checkpoint for multiple consecutive epochs.
    pub checkpoints: Vec<Checkpoint>,
    // Shard headers published but not included in the main chain.
    // The latter one in the list is the fresher.
    pub shard_header_pool: Vec<SignedShardHeader>,
}

impl BeaconChain {
    pub fn new() -> Self {
        Self {
            slot: GENESIS_SLOT,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            blocks: Vec::new(),
            states: Vec::new(),
            checkpoints: Vec::new(),
            shard_header_pool: Vec::new(),
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
        self.shard_header_pool.push(header);
    }

    /// Process of a slot.
    /// Extend the main chain by a newly proposed block.
    pub fn process_slot(&mut self, params: &BeaconSimulationParams) {
        // The parent epoch's checkpoint must be defined from epoch 1.
        if compute_epoch_at_slot(self.slot) > GENESIS_EPOCH {
            let current_epoch = compute_epoch_at_slot(self.slot);
            let parent_epoch = current_epoch - 1;
            // The case when parent epoch's checkpoint is not defined yet.
            if self.checkpoints.len() < parent_epoch as usize + 1 {
                // If there is no checkpoint defined, use the genesis parent root.
                // Otherwise, copy the grandparent epoch's checkpoint.
                let root: Root = if self.checkpoints.is_empty() {
                    GENESIS_PARENT_ROOT
                } else {
                    self.checkpoints.last().unwrap().root
                };
                self.checkpoints.push(Checkpoint {
                    epoch: parent_epoch, 
                    root: root,
                })
            }
            assert!((self.checkpoints.len() == parent_epoch as usize + 1) | (self.checkpoints.len() == current_epoch as usize + 1));    
        }
        
        if params.beacon_block_proposed {
            // Propose a new beacon block.
            self.append_new_block_to_chain(params.shard_headers_included, params.shard_headers_confirmed);
        }
        if params.beacon_chain_finalized {
            // Finalize a new checkpoint.
            self.progress_consensus();
        }
        self.slot += 1;
    }    

    /// Create new beacon state.
    /// This function is called before a block is proposed.
    fn create_new_state(&self, new_shard_headers: &Vec<SignedShardHeader>, shard_headers_confirmed: bool) -> BeaconState {
        let mut new_pending_shard_headers: Vec<PendingShardHeader> =
            new_shard_headers.iter().map(|signed_header| PendingShardHeader::from_signed_shard_header(&signed_header)).collect();        
        let mut current_pending_shard_headers: Vec<PendingShardHeader> = 
            if self.is_checkpoint_slot() {
                // Refresh current_pending_shard_headers at a checkpoint. 
                Vec::new()
            } else {
                // Otherwise, inherit the pending shard headers from the previous block.
                VariableList::into(self.states.last().unwrap().current_epoch_pending_shard_headers.clone())
            };
        current_pending_shard_headers.append(&mut new_pending_shard_headers);
        if shard_headers_confirmed {
            for header in &mut current_pending_shard_headers {
                header.confirmed = true;
            }
        }

        return BeaconState {
            slot: self.slot,
            finalized_checkpoint: self.finalized_checkpoint.clone(),
            current_epoch_pending_shard_headers: VariableList::from(current_pending_shard_headers), 
        }
    }
    
    /// Create a new block and append to the main chain.
    fn append_new_block_to_chain(&mut self, shard_headers_included: bool, shard_headers_confirmed: bool) {
        let new_shard_headers: Vec<SignedShardHeader>;
        if shard_headers_included {
            // If the number of headers in the pool exeeds the limit, select from the older headers.
            if self.shard_header_pool.len() > MAX_SHARD_HEADERS_PER_BLOCK {
                new_shard_headers = self.shard_header_pool[..MAX_SHARD_HEADERS_PER_BLOCK].to_vec();
                // Keep the fresher headers that are not selected in the pool.
                self.shard_header_pool = self.shard_header_pool[MAX_SHARD_HEADERS_PER_BLOCK..].to_vec();
            } else {
                new_shard_headers = self.shard_header_pool.clone();
                // All the published headers are included in the new beacon block.
                self.shard_header_pool.clear();
            }
        } else {
            new_shard_headers = Vec::new();
        }

        // Create the new beacon state.
        let state = self.create_new_state(&new_shard_headers, shard_headers_confirmed);
        self.states.push(state.clone());

        let parent_root = if self.blocks.is_empty() {
            GENESIS_PARENT_ROOT
        } else {
            self.blocks.last().unwrap().header().root()
        };

        let new_block = BeaconBlock {
            slot: self.slot,
            parent_root: parent_root,
            state_root: self.states.last().unwrap().root(),
            shard_headers: VariableList::from(new_shard_headers),
        };

        // Define the new block as the checkpoint if necessary.
        if self.is_checkpoint_slot() {
            self.checkpoints.push(Checkpoint {
                epoch: compute_epoch_at_slot(self.slot), 
                root: new_block.header().root(),
            })
        }

        self.blocks.push(new_block);
    }

    /// Progress consensus.
    /// Assumption: Checkpoints can be finalized only in the grandchild epochs.
    fn progress_consensus(&mut self) {
        if compute_epoch_at_slot(self.slot) < 2 {
            return
        }
        let finalized_epoch = compute_epoch_at_slot(self.slot) - 2;
        if (self.finalized_checkpoint == Checkpoint::genesis_finalized_checkpoint()) | (self.finalized_checkpoint.epoch < finalized_epoch) {
            self.finalized_checkpoint = self.checkpoints[finalized_epoch as usize].clone();
        }
    }

    // Whether or not we should propose a beacon block as the current epoch's checkpoint.
    // In other words, whether or not it is the first time to create a beacon block in the current epoch.
    fn is_checkpoint_slot(&self) -> bool {
        // Whether or not there is no checkpoint defined for the current epoch.
        self.checkpoints.len() < compute_epoch_at_slot(self.slot) as usize + 1
    }
}
