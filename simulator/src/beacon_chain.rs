use crate::*;
use simulation_params::BeaconSimulationParams;
use std::cmp;

/// Beacon chain consensus
pub struct BeaconChain {
    // The slot to be processed.
    // Note: The last processed slot is `self.slot - 1`.
    pub slot: Slot,
    // The latest beacon state (defined even for a slot without beacon block).
    pub state: BeaconState,
    // The latest finalized checkpoint.
    // Note: This is off-chain finality, not the finality verified in the beacon state.
    pub finalized_checkpoint: Checkpoint,
    // Beacon blocks in the main chain.
    // Note: Slots can be "skipped" i.e., there can be slots without beacon block proposal.
    // Assumption: No reorg or equivocation. (At most one block exists for each slot.)
    pub blocks: Vec<BeaconBlock>,
    // Beacon state at block of the main chain.
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
            state: BeaconState::genesis_state(),
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
    pub fn process_slot(&mut self, params: &BeaconSimulationParams) {
        if params.beacon_block_proposed {
            // Propose a new beacon block.
            // Shard shard headers to be included in the new beacon block.
            let (included_previous_epoch_shard_headers, mut included_current_epoch_shard_headers) =
                self.select_included_shard_headers(params.shard_headers_included); 
            // Update the state for the new block.
            self.update_state_for_new_block(&included_previous_epoch_shard_headers,
                &included_current_epoch_shard_headers, params.shard_headers_confirmed);

            let mut included_shard_headers = included_previous_epoch_shard_headers;
            included_shard_headers.append(&mut included_current_epoch_shard_headers);    
            // Append the new block to the chain.
            self.append_new_block_to_chain(included_shard_headers);
        }
        if params.beacon_chain_finalized {
            // Finalize a new checkpoint.
            self.progress_consensus();
        }
        if (self.slot + 1) % SLOTS_PER_EPOCH == 0 {
            self.process_epoch();
        }
        // Move to the slot to be processed.
        self.slot += 1;       
        self.state.slot = self.slot;
    }    

    // Process at the end of an epoch.
    fn process_epoch(&mut self){
        self.update_shard_gasprice();
        // Inherit the current pending shard headers to the next epoch.
        self.state.previous_epoch_pending_shard_headers = self.state.current_epoch_pending_shard_headers.clone();
        // Reset the current pending shard headers.
        self.state.current_epoch_pending_shard_headers = VariableList::from(Vec::new());

        // Reset the shard headers pool.
        self.previous_epoch_shard_header_pool = self.current_epoch_shard_header_pool.clone();
        self.current_epoch_shard_header_pool.clear();
    }

    /// Update shard gasprice.
    fn update_shard_gasprice(&mut self) {
        if compute_epoch_at_slot(self.slot) == GENESIS_EPOCH {
            return
        }
        let mut new_gasprice = self.state.shard_gasprice;
        let previous_epoch_start_slot = compute_start_slot_at_epoch(compute_epoch_at_slot(self.slot) - 1);
        for slot in previous_epoch_start_slot..previous_epoch_start_slot + SLOTS_PER_EPOCH {
            for shard in 0..SHARD_NUM as Shard {
                let confirmed_candidates: Vec<PendingShardHeader> = self.state.previous_epoch_pending_shard_headers.iter().filter(
                    |header| (header.slot, header.shard, header.confirmed) == (slot, shard, true)).cloned().collect();
                if confirmed_candidates.is_empty() {
                    continue;
                }
                let candidate = confirmed_candidates.get(0).unwrap();
                // Track updated gas price
                new_gasprice = Self::compute_updated_gasprice(
                    new_gasprice,
                    candidate.commitment.length,
                )
            }
        }
        self.state.shard_gasprice = new_gasprice
    }

    /// Update the pending shard headers in the beacon state.
    /// Store the shard headers included in the new beacon block in the state.
    fn update_pending_shard_headers(&mut self, included_previous_epoch_shard_headers: &Vec<SignedShardHeader>, 
        included_current_epoch_shard_headers: &Vec<SignedShardHeader>, shard_headers_confirmed: bool){
        for signed_header in included_previous_epoch_shard_headers.iter() {
            self.state.previous_epoch_pending_shard_headers.push(PendingShardHeader::from_signed_shard_header(&signed_header)).unwrap();
        }
        for signed_header in included_current_epoch_shard_headers.iter() {
            self.state.current_epoch_pending_shard_headers.push(PendingShardHeader::from_signed_shard_header(&signed_header)).unwrap();
        }

        if shard_headers_confirmed {
            for header in self.state.previous_epoch_pending_shard_headers.iter_mut() {
                header.confirmed = true;
            }
            for header in self.state.current_epoch_pending_shard_headers.iter_mut() {
                header.confirmed = true;
            }
        }
    }

    /// Update the beacon state for the new beacon block.
    fn update_state_for_new_block(&mut self, included_previous_epoch_shard_headers: &Vec<SignedShardHeader>, 
        included_current_epoch_shard_headers: &Vec<SignedShardHeader>, shard_headers_confirmed: bool) {
        self.update_pending_shard_headers(included_previous_epoch_shard_headers,
            included_current_epoch_shard_headers, shard_headers_confirmed);

        // Assumption: A new beacon block always include the attestations of the latest finalized checkpoint.
        self.state.finalized_checkpoint = self.finalized_checkpoint.clone();

    }

    /// Create a new block and append to the main chain.
    fn append_new_block_to_chain(&mut self, included_shard_headers: Vec<SignedShardHeader>) {
        let new_block = BeaconBlock {
            slot: self.slot,
            parent_root: if self.is_first_block_proposal() {
                            GENESIS_PARENT_ROOT
                        } else {
                            self.blocks.last().unwrap().header().root()
                        },
            state_root: self.state.root(),
            shard_headers: VariableList::from(included_shard_headers),
        };

        // Define checkpoints if necessary.
        // Define the same block for multiple consecutive epochs without beacon block proposal.
        // Ref: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/fork-choice.md#get_ancestor
        while self.is_checkpoint_missing() {
            self.checkpoints.push(Checkpoint {
                epoch: self.checkpoints.len() as Epoch, 
                root: new_block.header().root(),
            })
        }
        assert!(self.checkpoints.len() == compute_epoch_at_slot(self.slot) as usize + 1);

        // Store the new block.
        self.blocks.push(new_block);
        // Store the state of the new block.
        self.states.push(self.state.clone());
    }

    /// Shard shard headers to be included in the current slot's beacon block.
    fn select_included_shard_headers(&mut self, shard_headers_included: bool) -> (Vec<SignedShardHeader>, Vec<SignedShardHeader>) {
        let mut included_previous_epoch_shard_headers: Vec<SignedShardHeader> = Vec::new();
        let mut included_current_epoch_shard_headers: Vec<SignedShardHeader> = Vec::new();
        if shard_headers_included {
            // If the number of headers in the pool exceeds the limit, select from the older headers.
            if self.previous_epoch_shard_header_pool.len() > MAX_SHARD_HEADERS as usize {
                // Keep the fresher headers that are not selected in the pool.
                included_previous_epoch_shard_headers = self.previous_epoch_shard_header_pool.drain(..MAX_SHARD_HEADERS as usize).collect();
            } else {
                included_previous_epoch_shard_headers = self.previous_epoch_shard_header_pool.clone();
                let max_current_epoch_shard_headers = MAX_SHARD_HEADERS as usize - self.previous_epoch_shard_header_pool.len();
                self.previous_epoch_shard_header_pool.clear();
                if self.current_epoch_shard_header_pool.len() > max_current_epoch_shard_headers {
                    // Keep the fresher headers that are not selected in the pool.
                    included_current_epoch_shard_headers = self.current_epoch_shard_header_pool.drain(..max_current_epoch_shard_headers).collect();
                } else {
                    included_current_epoch_shard_headers = self.current_epoch_shard_header_pool.clone();
                    // All the published headers are included in the new beacon block.
                    self.current_epoch_shard_header_pool.clear();    
                }
            }
        }
        return (included_previous_epoch_shard_headers, included_current_epoch_shard_headers)
    }

    /// Progress consensus (off-chain finality).
    /// The `finalized_checkpoint` in the beacon state is not updated in a slot without block proposal,
    /// since attestations to finalized the checkpoint are not included in the chain yet.
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

    /// Whether or not it is the first time to propose a beacon block.
    fn is_first_block_proposal(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Whether or not checkpoints are not enough.
    fn is_checkpoint_missing(&self) -> bool {        
        self.checkpoints.len() < compute_epoch_at_slot(self.slot) as usize + 1
    }
}
