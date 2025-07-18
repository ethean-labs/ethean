//! API type definitions
//!
//! Common types and structures used across the API.

use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub execution_optimistic: bool,
    pub finalized: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            execution_optimistic: false,
            finalized: true,
            data,
        }
    }

    pub fn optimistic(data: T) -> Self {
        Self {
            execution_optimistic: true,
            finalized: false,
            data,
        }
    }

    pub fn success(data: T) -> Self {
        Self::new(data)
    }
}

/// Standard error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub stacktraces: Option<Vec<String>>,
}

impl ErrorResponse {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            stacktraces: None,
        }
    }

    pub fn with_stacktrace(mut self, stacktrace: Vec<String>) -> Self {
        self.stacktraces = Some(stacktrace);
        self
    }
}

/// Genesis information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Genesis {
    pub genesis_time: String,
    pub genesis_validators_root: String,
    pub genesis_fork_version: String,
}

impl Default for Genesis {
    fn default() -> Self {
        Self {
            genesis_time: "1606824000".to_string(),
            genesis_validators_root: "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95".to_string(),
            genesis_fork_version: "0x00000000".to_string(),
        }
    }
}

/// Fork information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Fork {
    pub previous_version: String,
    pub current_version: String,
    pub epoch: String,
}

impl Default for Fork {
    fn default() -> Self {
        Self {
            previous_version: "0x00000000".to_string(),
            current_version: "0x01000000".to_string(),
            epoch: "0".to_string(),
        }
    }
}

/// Validator information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Validator {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: String,
    pub slashed: bool,
    pub activation_eligibility_epoch: String,
    pub activation_epoch: String,
    pub exit_epoch: String,
    pub withdrawable_epoch: String,
}

/// Validator status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorStatus {
    PendingInitialized,
    PendingQueued,
    ActiveOngoing,
    ActiveExiting,
    ActiveSlashed,
    ExitedUnslashed,
    ExitedSlashed,
    WithdrawalPossible,
    WithdrawalDone,
}

/// Block header
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockHeader {
    pub message: BlockHeaderMessage,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockHeaderMessage {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

/// Signed beacon block
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconBlock {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body: BeaconBlockBody,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconBlockBody {
    pub randao_reveal: String,
    pub eth1_data: Eth1Data,
    pub graffiti: String,
    pub proposer_slashings: Vec<ProposerSlashing>,
    pub attester_slashings: Vec<AttesterSlashing>,
    pub attestations: Vec<Attestation>,
    pub deposits: Vec<Deposit>,
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
}

/// Eth1 data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Eth1Data {
    pub deposit_root: String,
    pub deposit_count: String,
    pub block_hash: String,
}

/// Proposer slashing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconBlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

/// Attester slashing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AttesterSlashing {
    pub attestation_1: IndexedAttestation,
    pub attestation_2: IndexedAttestation,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<String>,
    pub data: AttestationData,
    pub signature: String,
}

/// Attestation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Attestation {
    pub aggregation_bits: String,
    pub data: AttestationData,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AttestationData {
    pub slot: String,
    pub index: String,
    pub beacon_block_root: String,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Checkpoint {
    pub epoch: String,
    pub root: String,
}

/// Deposit
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Deposit {
    pub proof: Vec<String>,
    pub data: DepositData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositData {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: String,
    pub signature: String,
}

/// Voluntary exit
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VoluntaryExit {
    pub epoch: String,
    pub validator_index: String,
}

/// Committee assignment
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Committee {
    pub index: String,
    pub slot: String,
    pub validators: Vec<String>,
}

/// Validator duty
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AttesterDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub committee_index: String,
    pub committee_length: String,
    pub committees_at_slot: String,
    pub validator_committee_index: String,
    pub slot: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProposerDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub slot: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub validator_sync_committee_indices: Vec<String>,
}

/// Node information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeIdentity {
    pub peer_id: String,
    pub enr: String,
    pub p2p_addresses: Vec<String>,
    pub discovery_addresses: Vec<String>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeMetadata {
    pub seq_number: String,
    pub attnets: String,
    pub syncnets: String,
}

/// Peer information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Peer {
    pub peer_id: String,
    pub enr: Option<String>,
    pub last_seen_p2p_address: String,
    pub state: PeerState,
    pub direction: PeerDirection,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PeerState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PeerDirection {
    Inbound,
    Outbound,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: HealthStatus,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ready,
    Syncing,
    NotReady,
}

/// Version information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionResponse {
    pub version: String,
}

/// Sync status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncStatus {
    pub head_slot: String,
    pub sync_distance: String,
    pub is_syncing: bool,
    pub is_optimistic: bool,
    pub el_offline: bool,
}

/// Fork schedule
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkScheduleItem {
    pub previous_version: String,
    pub current_version: String,
    pub epoch: String,
}

/// Spec configuration
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecResponse {
    pub data: std::collections::HashMap<String, String>,
}

/// Deposit contract information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositContract {
    pub chain_id: String,
    pub address: String,
}

/// BLS to execution change
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SignedBLSToExecutionChange {
    pub message: BLSToExecutionChange,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BLSToExecutionChange {
    pub validator_index: String,
    pub from_bls_pubkey: String,
    pub to_execution_address: String,
}

/// Sync committee message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncCommitteeMessage {
    pub slot: String,
    pub beacon_block_root: String,
    pub validator_index: String,
    pub signature: String,
}

/// Sync committee contribution
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncCommitteeContribution {
    pub slot: String,
    pub beacon_block_root: String,
    pub subcommittee_index: String,
    pub aggregation_bits: String,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::new("test_data");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_data"));
        assert!(json.contains("execution_optimistic"));
        assert!(json.contains("finalized"));
    }

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new(404, "Not found");
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "Not found");
        assert!(error.stacktraces.is_none());
    }

    #[test]
    fn test_validator_status_serialization() {
        let status = ValidatorStatus::ActiveOngoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active_ongoing\"");
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Ready;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"ready\"");
    }
}
