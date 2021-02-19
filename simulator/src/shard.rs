use crate::*;
use simulation_params::ShardSimulationParams;

/// Shard's data market.
pub struct ShardDataMarket {
    // The slot to be processed.
    // Note: The last processed slot is `self.slot - 1`.
    pub slot: Slot,
    pub shard: Shard,
    // Published bids of each slot that are not selected by proposers yet. 
    pub bid_pool: Vec<Vec<Bid>>,
    // Proposed shard headers of each slot.
    // Assumption: No equivocation. 
    pub proposed_headers: Vec<Option<SignedShardHeader>>,
}

impl ShardDataMarket {
    pub fn new(shard: Shard) -> Self {
        Self {
            slot: GENESIS_SLOT,
            shard,
            bid_pool: Vec::new(),
            proposed_headers: Vec::new(),
        }
    }

    /// Publish a bid in the shard's subnet.
    pub fn publish_bid(&mut self, bid: Bid) {
        while self.bid_pool.len() <= bid.slot as usize {
            self.bid_pool.push(Vec::new());
        }
        self.bid_pool[bid.slot as usize].push(bid);
    }

    /// Process of a slot.
    pub fn process_slot(&mut self, params: &ShardSimulationParams) {
        while self.bid_pool.len() <= self.slot as usize {
            self.bid_pool.push(Vec::new());
        }

        if params.blob_proposed {
            self.propose_blob(params.data_included);
        } else {
            self.proposed_headers.push(None);
        }
        assert_eq!(self.slot as usize + 1, self.proposed_headers.len());
        self.slot += 1;
    }

    /// Propose a shard blob.
    /// Pick up the bid with the highest fee.
    /// Note: For now, we don't simulate with `ShardBlob`, and use bids directly.
    fn propose_blob(&mut self, data_included: bool) {
        let commitment: DataCommitment;
        if !data_included | self.bid_pool[self.slot as usize].is_empty() {
            commitment = DataCommitment::default();
        } else {
            // Sort bids in ascending order by fee
            self.bid_pool[self.slot as usize].sort_by(|a, b| a.fee.cmp(&b.fee));
            commitment = self.bid_pool[self.slot as usize].pop().unwrap().commitment;
        }
        // Note: For now, use dummy data to replace BLS signature.
        self.proposed_headers.push(
            Some(SignedShardHeader::dummy_from_header(
                ShardHeader {
                    slot: self.slot,
                    shard: self.shard,
                    commitment: commitment,
            })));
    } 

}
