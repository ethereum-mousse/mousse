//! Custom types in the Eth2 system
//! Ref: https://github.com/ethereum/eth2.0-specs/blob/849837a07d1e3dbf7c75d71b14034c10315f6341/specs/phase1/beacon-chain.md
use crate::eth2_config::*;
pub use ethereum_types::{H256, U256};
use serde_derive::{Deserialize, Serialize};
pub use ssz_types::{typenum, VariableList};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

big_array! { BigArray; }

/// u64.
pub type Slot = u64;
/// u64.
pub type Epoch = u64;
/// u64.
pub type Shard = u64;
/// u64.
pub type Gwei = u64;
/// H256.
pub type Root = H256;
/// [u8; 96].
const BLS_SIGNATURE_BYTE_LEN: usize = 96;
pub type BLSSignature = [u8; BLS_SIGNATURE_BYTE_LEN];
/// u64.
/// TODO: This should be bytes48. We leave this fix to avoid SSZ implementation.
/// Ref: https://github.com/sigp/lighthouse/blob/v1.0.6/crypto/bls/src/generic_public_key_bytes.rs#L22
pub type BLSCommitment = u64;
pub const BYTES_PER_POINT: usize = 31;
/// Variable list of uint256. The length is MAX_SAMPLES_PER_BLOCK.
/// TODO: Fix the length.
pub type BlobData = VariableList<U256, typenum::U2048>;

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Deserialize, Serialize)]
pub struct DataCommitment {
    pub point: BLSCommitment,
    pub length: u64,
}

impl DataCommitment {
    pub fn dummy_from_bytes(bytes: &Vec<u8>) -> Self {
        Self {
            // TODO: Use the real KZG commitment.
            point: calculate_hash(bytes),
            // Each point is 31 bytes.
            length: (bytes.len() as f64 / BYTES_PER_POINT as f64).ceil() as u64,
        }
    }
}

// Ref: https://doc.rust-lang.org/std/hash/index.html#examples
pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// TODO: Replace this with SSZ root.
fn root<T: Hash>(t: &T) -> Root {
    let hash: &mut Vec<u8> = &mut u64::to_le_bytes(calculate_hash(t)).to_vec();
    let hash2: &[u8; 8] = &u64::to_le_bytes(calculate_hash(hash));
    let hash3: &[u8; 8] = &u64::to_le_bytes(calculate_hash(hash2));
    let hash4: &[u8; 8] = &u64::to_le_bytes(calculate_hash(hash3));
    hash.extend_from_slice(hash2);
    hash.extend_from_slice(hash3);
    hash.extend_from_slice(hash4);
    H256::from_slice(hash)
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
            signature: signature,
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
}

#[derive(Clone)]
pub struct ShardBlob {
    pub slot: Slot,
    pub shard: Shard,
    pub data: BlobData,
}

pub fn compute_epoch_at_slot(slot: Slot) -> Epoch {
    slot / SLOTS_PER_EPOCH as Epoch
}

pub fn compute_start_slot_at_epoch(epoch: Epoch) -> Slot {
    epoch * SLOTS_PER_EPOCH as Epoch
}
