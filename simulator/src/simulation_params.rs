use crate::*;

/// Settings of the shard simulation of a slot.
#[derive(Debug)]
pub struct ShardSimulationParams {
    /// Whether or not a shard blob is proposed.
    /// Assumption: If a shard blob is proposed, its header is published on the global subnet.
    pub blob_proposed: bool,
    /// Whether or not data specified by a bid is included in a shard blob.
    pub data_included: bool,
}

impl ShardSimulationParams {
    /// Happy case of the shard.
    pub fn happy() -> Self {
        Self {
            blob_proposed: true,
            data_included: true,
        }
    }

    /// Everything fails.
    pub fn all_failure() -> Self {
        Self::no_blob_proposal()
    }

    /// No shard blob is proposed.
    /// Note: If no shard blob is proposed, no data is included.
    pub fn no_blob_proposal() -> Self {
        Self {
            blob_proposed: false,
            data_included: false,
        }
    }

    /// No data is included.
    pub fn no_data_inclusion() -> Self {
        Self {
            blob_proposed: true,
            data_included: false,
        }
    }
}

/// Settings of the beacon chain simulation of a slot.
#[derive(Debug)]
pub struct BeaconSimulationParams {
    /// Whether or not a beacon block is proposed.
    pub beacon_block_proposed: bool,
    /// Whether or not the grandparent epoch's checkpoint gets finalized (if it is not finalized yet).
    /// Assumption: Checkpoints can be finalized only in the grandchild epochs.
    /// TODO: Simulate the best case where the parent epoch's checkpoint gets finalized at the 2/3 of the current epoch.
    pub beacon_chain_finalized: bool,
    /// Whether or not non-included headers of each shard are included.
    /// Assumption: All the skipped headers are included in a beacon block later at the same time.
    /// TODO: Allow shard-by-shard configuration.
    /// TODO: Limit the number of headers to be included to `MAX_SHARD_HEADERS_PER_SHARD`.
    /// Ref: https://github.com/ethereum/eth2.0-specs/blob/069fbd7b910410ef47a9fb7a1e4839ac32f39929/specs/phase1/beacon-chain.md#configuration
    pub shard_headers_included: bool,
    /// Whether or not included and non-confirmed headers of each shard are confirmed.
    /// Assumption: All the non-confirmed headers are confirmed later at the same time.
    /// TODO: Allow shard-by-shard configuration.
    pub shard_headers_confirmed: bool,
}

impl BeaconSimulationParams {
    /// Happy case of the beacon chain.
    pub fn happy() -> Self {
        Self {
            beacon_block_proposed: true,
            beacon_chain_finalized: true,
            shard_headers_included: true,
            shard_headers_confirmed: true,
        }
    }

    /// Everything fails.
    pub fn all_failure() -> Self {
        Self {
            beacon_block_proposed: false,
            beacon_chain_finalized: false,
            shard_headers_included: false,
            shard_headers_confirmed: false,
        }
    }

    /// No beacon block is proposed.
    /// Note: If there is no beacon block proposed, attestations and shard headers cannot be included in the beacon chain.
    pub fn no_block_proposal() -> Self {
        Self {
            beacon_block_proposed: false,
            beacon_chain_finalized: true,
            shard_headers_included: false,
            shard_headers_confirmed: false,
        }
    }

    /// No checkpoint gets finalized.
    pub fn no_chain_finality() -> Self {
        Self {
            beacon_block_proposed: true,
            beacon_chain_finalized: false,
            shard_headers_included: true,
            shard_headers_confirmed: true,
        }
    }

    /// None of the headers of any shard gets included.
    pub fn no_shard_header_inclusion() -> Self {
        Self {
            beacon_block_proposed: true,
            beacon_chain_finalized: false,
            shard_headers_included: false,
            shard_headers_confirmed: true,
        }
    }

    /// None of the headers of any shard gets confirmed.    
    pub fn no_shard_header_confirmation() -> Self {
        Self {
            beacon_block_proposed: true,
            beacon_chain_finalized: false,
            shard_headers_included: true,
            shard_headers_confirmed: false,
        }
    }
}

/// Settings of the simulation of a slot.
/// TODO: Add more complicated failure cases.
#[derive(Debug)]
pub struct SimulationParams {
    /// Settings of the beacon chain simulation of a slot.
    pub beacon_params: BeaconSimulationParams,
    /// Settings of the shard simulation of a slot.
    pub shard_params: Vec<ShardSimulationParams>,
}

impl SimulationParams {
    /// Happy case.
    pub fn happy() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::happy(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::happy())
                .collect(),
        }
    }

    /// Everything fails.
    pub fn all_failure() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::all_failure(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::all_failure())
                .collect(),
        }
    }

    /// No data gets included in any shard.
    pub fn no_shard_data_inclusion() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::happy(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::no_data_inclusion())
                .collect(),
        }
    }

    /// No shard blob is proposed in any shard.
    pub fn no_shard_blob_proposal() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::happy(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::no_blob_proposal())
                .collect(),
        }
    }

    /// No shard header is included in any shard.
    pub fn no_shard_header_inclusion() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::no_shard_header_inclusion(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::happy())
                .collect(),
        }
    }

    /// No shard header is confirmed in any shard.
    pub fn no_shard_header_confirmation() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::no_shard_header_confirmation(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::happy())
                .collect(),
        }
    }

    /// No checkpoint gets finalized.
    pub fn no_beacon_chain_finality() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::no_chain_finality(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::happy())
                .collect(),
        }
    }

    /// No beacon block is proposed.
    pub fn no_beacon_block_proposal() -> Self {
        Self {
            beacon_params: BeaconSimulationParams::no_block_proposal(),
            shard_params: (0..SHARD_NUM)
                .map(|_| ShardSimulationParams::happy())
                .collect(),
        }
    }

    /// Fails randomly.
    pub fn random() -> Self {
        let rn: usize = rand::thread_rng().gen();
        match rn % 8 {
            0 => Self::happy(),
            1 => Self::all_failure(),
            2 => Self::no_shard_data_inclusion(),
            3 => Self::no_shard_blob_proposal(),
            4 => Self::no_shard_header_inclusion(),
            5 => Self::no_shard_header_confirmation(),
            6 => Self::no_beacon_chain_finality(),
            7 => Self::no_beacon_block_proposal(),
            _ => Self::happy(),
        }
    }
}
