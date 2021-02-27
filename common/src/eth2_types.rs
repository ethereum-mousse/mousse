//! Custom types in the Eth2 system
//! Ref: https://github.com/ethereum/eth2.0-specs/blob/849837a07d1e3dbf7c75d71b14034c10315f6341/specs/phase1/beacon-chain.md
use crate::eth2_config::*;
use crate::eth2_utils::{calculate_hash, root};
pub use ethereum_types::{H256, U256};
use serde_derive::{Deserialize, Serialize};
pub use ssz_types::{typenum, VariableList};
use std::hash::{Hash, Hasher};

big_array! { BigArray; }

const BLS_SIGNATURE_BYTE_LEN: usize = 96;
const BLS_COMMITMENT_BYTE_LEN: usize = 48;
pub const BYTES_PER_POINT: usize = 31;

/// u64.
pub type Slot = u64;
/// u64.
pub type Epoch = u64;
/// u64.
pub type Shard = u64;
/// Alias for `Shard`.
pub type ShardId = Shard;
/// u64.
pub type Gwei = u64;
/// H256.
pub type Root = H256;
/// [u8; 96].
pub type BLSSignature = [u8; BLS_SIGNATURE_BYTE_LEN];
/// [u8; 48].
pub type BLSCommitment = [u8; BLS_COMMITMENT_BYTE_LEN];
/// U256.
/// Call this `FieldElement` instead of `BLSPoint`.
/// Ref: https://github.com/ethereum/eth2.0-specs/pull/2172#discussion_r550884186
pub type FieldElement = U256;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Root,
}

impl Checkpoint {
    pub fn genesis_finalized_checkpoint() -> Self {
        Self {
            epoch: GENESIS_EPOCH,
            root: GENESIS_PARENT_ROOT,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct DataCommitment {
    #[serde(with = "BigArray")]
    pub point: BLSCommitment,
    pub length: u64,
}

impl Default for DataCommitment {
    fn default() -> Self {
        Self {
            point: [0; BLS_COMMITMENT_BYTE_LEN],
            length: 0,
        }
    }
}

impl DataCommitment {
    /// Generate a dummy commitment based on the data's hash.
    /// TODO: Use the real KZG commitment.
    #[allow(clippy::ptr_arg)]
    pub fn dummy_from_bytes(bytes: &[u8]) -> Self {
        let mut hash: u64 = calculate_hash(&bytes.to_vec());
        let mut dummy_sig: Vec<u8> = Vec::new();
        for _ in 0..BLS_COMMITMENT_BYTE_LEN / 8 {
            hash = calculate_hash(&hash);
            dummy_sig.extend_from_slice(&u64::to_le_bytes(hash));
        }
        assert_eq!(BLS_COMMITMENT_BYTE_LEN, dummy_sig.len());
        let mut point: [u8; BLS_COMMITMENT_BYTE_LEN] = [0; BLS_COMMITMENT_BYTE_LEN];
        for (i, v) in dummy_sig.iter().enumerate() {
            point[i] = *v;
        }

        Self {
            point,
            // Each point is 31 bytes.
            length: (bytes.len() as f64 / BYTES_PER_POINT as f64).ceil() as u64,
        }
    }
}

/// `degree_proof` field is omitted.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct ShardHeader {
    pub slot: Slot,
    pub shard: Shard,
    pub commitment: DataCommitment,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct SignedShardHeader {
    pub message: ShardHeader,
    #[serde(with = "BigArray")]
    pub signature: BLSSignature,
}

impl SignedShardHeader {
    /// Generate a signed shard header with a dummy signature.
    /// The dummy signature is based on the header's hash, so deterministic.
    /// TODO: Use the real BLS signature.
    pub fn dummy_from_header(header: ShardHeader) -> Self {
        let mut hash: u64 = calculate_hash(&header);
        let mut dummy_sig: Vec<u8> = Vec::new();
        for _ in 0..BLS_SIGNATURE_BYTE_LEN / 8 {
            hash = calculate_hash(&hash);
            dummy_sig.extend_from_slice(&u64::to_le_bytes(hash));
        }
        assert_eq!(BLS_SIGNATURE_BYTE_LEN, dummy_sig.len());
        let mut signature: [u8; BLS_SIGNATURE_BYTE_LEN] = [0; BLS_SIGNATURE_BYTE_LEN];
        for (i, v) in dummy_sig.iter().enumerate() {
            signature[i] = *v;
        }
        Self {
            message: header,
            signature,
        }
    }
}

/// `votes` field is omitted.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct PendingShardHeader {
    pub slot: Slot,
    pub shard: Shard,
    pub commitment: DataCommitment,
    pub root: Root,
    pub confirmed: bool,
}

impl PendingShardHeader {
    pub fn from_signed_shard_header(signed_header: &SignedShardHeader) -> Self {
        let header = &signed_header.message;
        PendingShardHeader {
            slot: header.slot,
            shard: header.shard,
            commitment: header.commitment.clone(),
            root: root(signed_header),
            // Default is `false`.
            confirmed: false,
        }
    }
}

/// Only necessary fields are defined.
#[derive(Clone, Deserialize, Serialize)]
pub struct BeaconBlock {
    pub slot: Slot,
    pub parent_root: Root,
    pub state_root: Root,
    /// The length is MAX_SHARD_HEADERS (= SHARD_NUM * MAX_SHARD_HEADERS_PER_SHARD).
    pub shard_headers: VariableList<SignedShardHeader, typenum::U256>,
}

/// Implement `Hash` manually to handle `VariableList`.
impl Hash for BeaconBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.parent_root.hash(state);
        self.state_root.hash(state);
        let headers: Vec<SignedShardHeader> = VariableList::into(self.shard_headers.clone());
        headers.hash(state);
    }
}

impl BeaconBlock {
    pub fn header(&self) -> BeaconBlockHeader {
        let headers: &Vec<SignedShardHeader> = &VariableList::into(self.shard_headers.clone());
        BeaconBlockHeader {
            slot: self.slot,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: root(headers),
        }
    }
}

#[derive(Hash, Clone)]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

impl BeaconBlockHeader {
    pub fn root(&self) -> Root {
        root(&self)
    }
}

/// Only necessary fields are defined.
#[derive(Clone, Deserialize, Serialize)]
pub struct BeaconState {
    pub slot: Slot,
    pub finalized_checkpoint: Checkpoint,
    /// The length is MAX_SHARD_HEADERS * SLOTS_PER_EPOCH.
    pub previous_epoch_pending_shard_headers: VariableList<PendingShardHeader, typenum::U8192>,
    pub current_epoch_pending_shard_headers: VariableList<PendingShardHeader, typenum::U8192>,
    pub shard_gasprice: Gwei,
}

/// Implement `Hash` manually to handle `VariableList`.
impl Hash for BeaconState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.finalized_checkpoint.hash(state);
        let headers: Vec<PendingShardHeader> =
            VariableList::into(self.current_epoch_pending_shard_headers.clone());
        headers.hash(state);
    }
}

impl BeaconState {
    pub fn root(&self) -> Root {
        root(&self)
    }

    pub fn genesis_state() -> Self {
        Self {
            slot: GENESIS_SLOT,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            shard_gasprice: INIT_SHARD_GASPRICE,
        }
    }
}

#[derive(Clone)]
pub struct ShardBlob {
    pub slot: Slot,
    pub shard: Shard,
    // The length is POINTS_PER_SAMPLE * MAX_SAMPLES_PER_BLOCK.
    pub data: VariableList<FieldElement, typenum::U16384>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_root() {
        let signed_headers: Vec<SignedShardHeader> = (0..SHARD_NUM * 2)
            .map(|num| {
                SignedShardHeader::dummy_from_header(ShardHeader {
                    slot: (num / SHARD_NUM) as Slot,
                    shard: (num % SHARD_NUM) as Shard,
                    commitment: generate_dummy_from_str(&format!(
                        "Slot {}, Shard {}",
                        num / SHARD_NUM,
                        num % SHARD_NUM
                    )),
                })
            })
            .collect();
        let state1 = BeaconState {
            slot: 0,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(
                signed_headers[..SHARD_NUM as usize]
                    .iter()
                    .map(|signed_header| {
                        PendingShardHeader::from_signed_shard_header(signed_header)
                    })
                    .collect::<Vec<PendingShardHeader>>(),
            ),
            shard_gasprice: 0,
        };
        let block1 = BeaconBlock {
            slot: 0,
            parent_root: H256::zero(),
            state_root: state1.root(),
            shard_headers: VariableList::from(signed_headers[..SHARD_NUM as usize].to_vec()),
        };
        let state2 = BeaconState {
            slot: 1,
            finalized_checkpoint: Checkpoint::genesis_finalized_checkpoint(),
            previous_epoch_pending_shard_headers: VariableList::from(Vec::new()),
            current_epoch_pending_shard_headers: VariableList::from(
                signed_headers[SHARD_NUM as usize..]
                    .iter()
                    .map(|signed_header| {
                        PendingShardHeader::from_signed_shard_header(signed_header)
                    })
                    .collect::<Vec<PendingShardHeader>>(),
            ),
            shard_gasprice: 0,
        };
        let block2 = BeaconBlock {
            slot: 1,
            parent_root: block1.header().root(),
            state_root: state2.root(),
            shard_headers: VariableList::from(signed_headers[SHARD_NUM as usize..].to_vec()),
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
                signed_headers[SHARD_NUM as usize..]
                    .iter()
                    .map(|signed_header| {
                        PendingShardHeader::from_signed_shard_header(signed_header)
                    })
                    .collect::<Vec<PendingShardHeader>>(),
            ),
            shard_gasprice: 0,
        };
        let another_block2 = BeaconBlock {
            slot: 1,
            parent_root: block1.header().root(),
            state_root: state2.root(),
            shard_headers: VariableList::from(signed_headers[SHARD_NUM as usize..].to_vec()),
        };
        assert_eq!(state2.root(), another_state2.root());
        assert_eq!(block2.header().root(), another_block2.header().root());
    }

    #[test]
    fn dummy_commitment() {
        check_dummy_from_string(String::from(""));
        check_dummy_from_string(String::from("hello"));
        compare_dummy_from_string(String::from("sharding"), String::from("sharding"));
        compare_dummy_from_string(String::from(""), String::from("Ethereum"));
        compare_dummy_from_string(String::from("Eth1"), String::from("Eth2"));
    }

    fn check_dummy_from_string(s: String) {
        let bytes = s.clone().into_bytes();
        let commitment = DataCommitment::dummy_from_bytes(&bytes);
        assert_eq!(
            (s.len() as f64 / BYTES_PER_POINT as f64).ceil() as u64,
            commitment.length
        );
    }

    fn generate_dummy_from_str(s: &str) -> DataCommitment {
        let bytes = s.as_bytes();
        DataCommitment::dummy_from_bytes(bytes)
    }

    fn compare_dummy_from_string(s1: String, s2: String) {
        let commitment1 = generate_dummy_from_str(&s1);
        let commitment2 = generate_dummy_from_str(&s2);
        if s1 == s2 {
            assert_eq!(commitment1, commitment2);
        } else {
            assert_ne!(commitment1, commitment2);
        }
    }

    #[test]
    fn dummy_signed_shard_header() {
        let header = ShardHeader {
            slot: 0,
            shard: 0,
            commitment: generate_dummy_from_str(&String::from("Ethreum")),
        };
        let signed_header1 = SignedShardHeader::dummy_from_header(header.clone());
        let signed_header2 = SignedShardHeader::dummy_from_header(header);
        // Dummy signature is deterministic.
        assert_eq!(signed_header1, signed_header2);
    }
}
