# Security Implementation Guide

## Overview

Comprehensive security implementation for Panro beacon chain client covering cryptographic protocols, network security, key management, attack prevention, and security hardening practices.

## Security Architecture

### Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Architecture                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Application │  │ Cryptographic│  │ Key Management      │ │
│  │ Security    │  │ Protocols    │  │ System             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Network     │  │ Input       │  │ Access              │ │
│  │ Security    │  │ Validation  │  │ Control             │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Infrastructure │ │ Monitoring │  │ Incident           │ │
│  │ Security      │  │ & Auditing │  │ Response            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Cryptographic Implementation

### BLS Signature Verification

```rust
use blst::{
    min_pk::{PublicKey, Signature, SecretKey, AggregateSignature, AggregatePublicKey},
    BLST_ERROR,
};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

pub struct BlsSignatureVerifier {
    domain_separator: [u8; 32],
    cached_public_keys: LruCache<ValidatorIndex, PublicKey>,
    signature_cache: LruCache<SignatureHash, bool>,
}

impl BlsSignatureVerifier {
    pub fn new(genesis_validators_root: Root, fork_version: ForkVersion) -> Self {
        let domain_separator = compute_domain_separator(genesis_validators_root, fork_version);
        
        Self {
            domain_separator,
            cached_public_keys: LruCache::new(NonZeroUsize::new(100000).unwrap()),
            signature_cache: LruCache::new(NonZeroUsize::new(10000).unwrap()),
        }
    }
    
    pub fn verify_block_signature(
        &mut self,
        block: &SignedBeaconBlock,
        proposer_public_key: &PublicKey,
    ) -> Result<bool, SignatureError> {
        // Create signing root
        let signing_root = self.compute_signing_root(
            &block.message,
            DomainType::BeaconProposer,
            block.message.slot.epoch(SLOTS_PER_EPOCH),
        )?;
        
        // Check signature cache
        let sig_hash = self.compute_signature_hash(&block.signature, &signing_root);
        if let Some(&cached_result) = self.signature_cache.get(&sig_hash) {
            return Ok(cached_result);
        }
        
        // Verify signature
        let result = self.verify_signature(
            &block.signature,
            &signing_root,
            proposer_public_key,
        )?;
        
        // Cache result
        self.signature_cache.put(sig_hash, result);
        
        Ok(result)
    }
    
    pub fn verify_attestation_signature(
        &mut self,
        attestation: &Attestation,
        committee_public_keys: &[PublicKey],
    ) -> Result<bool, SignatureError> {
        let data = &attestation.data;
        
        // Create signing root
        let signing_root = self.compute_signing_root(
            data,
            DomainType::BeaconAttester,
            data.target.epoch,
        )?;
        
        // Aggregate public keys based on aggregation bits
        let mut aggregate_pubkey = AggregatePublicKey::new();
        for (i, pubkey) in committee_public_keys.iter().enumerate() {
            if attestation.aggregation_bits.get_bit(i)? {
                aggregate_pubkey.add_public_key(pubkey);
            }
        }
        
        let aggregated_pubkey = aggregate_pubkey.to_public_key();
        
        // Verify aggregated signature
        self.verify_signature(
            &attestation.signature,
            &signing_root,
            &aggregated_pubkey,
        )
    }
    
    pub fn verify_aggregate_signature(
        &mut self,
        signatures: &[Signature],
        signing_roots: &[SigningRoot],
        public_keys: &[PublicKey],
    ) -> Result<bool, SignatureError> {
        if signatures.len() != signing_roots.len() || signatures.len() != public_keys.len() {
            return Err(SignatureError::MismatchedLengths);
        }
        
        // Aggregate signatures
        let mut aggregate_sig = AggregateSignature::new();
        for signature in signatures {
            aggregate_sig.add_signature(signature);
        }
        
        // Verify aggregate
        let msgs: Vec<&[u8]> = signing_roots.iter().map(|root| root.as_bytes()).collect();
        let pubkeys: Vec<&PublicKey> = public_keys.iter().collect();
        
        match aggregate_sig.aggregate_verify(&msgs, &pubkeys) {
            BLST_ERROR::BLST_SUCCESS => Ok(true),
            _ => Ok(false),
        }
    }
    
    fn verify_signature(
        &self,
        signature: &Signature,
        signing_root: &SigningRoot,
        public_key: &PublicKey,
    ) -> Result<bool, SignatureError> {
        match signature.verify(signing_root.as_bytes(), public_key) {
            BLST_ERROR::BLST_SUCCESS => Ok(true),
            BLST_ERROR::BLST_VERIFY_FAIL => Ok(false),
            error => Err(SignatureError::BlstError(error)),
        }
    }
    
    fn compute_signing_root<T: TreeHash>(
        &self,
        object: &T,
        domain_type: DomainType,
        epoch: Epoch,
    ) -> Result<SigningRoot, SignatureError> {
        let object_root = object.tree_hash_root();
        let domain = self.compute_domain(domain_type, epoch)?;
        
        let mut hasher = Sha256::new();
        hasher.update(object_root.as_bytes());
        hasher.update(domain.as_bytes());
        
        Ok(SigningRoot::from_slice(&hasher.finalize()))
    }
    
    fn compute_domain(
        &self,
        domain_type: DomainType,
        epoch: Epoch,
    ) -> Result<Domain, SignatureError> {
        let fork_version = self.get_fork_version(epoch)?;
        let fork_data_root = compute_fork_data_root(fork_version, self.domain_separator)?;
        
        let mut domain = [0u8; 32];
        domain[0..4].copy_from_slice(&domain_type.as_bytes());
        domain[4..32].copy_from_slice(&fork_data_root.as_bytes()[0..28]);
        
        Ok(Domain::from(domain))
    }
}

// Secure random number generation
pub struct SecureRng {
    rng: ChaCha20Rng,
}

impl SecureRng {
    pub fn new() -> Result<Self, CryptoError> {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed)?;
        
        Ok(Self {
            rng: ChaCha20Rng::from_seed(seed),
        })
    }
    
    pub fn generate_keypair(&mut self) -> (SecretKey, PublicKey) {
        let mut key_material = [0u8; 32];
        self.rng.fill_bytes(&mut key_material);
        
        let secret_key = SecretKey::key_gen(&key_material, &[]).unwrap();
        let public_key = secret_key.sk_to_pk();
        
        // Clear sensitive data
        key_material.zeroize();
        
        (secret_key, public_key)
    }
    
    pub fn generate_withdrawal_credentials(&mut self) -> WithdrawalCredentials {
        let mut credentials = [0u8; 32];
        credentials[0] = 0x01; // BLS withdrawal prefix
        self.rng.fill_bytes(&mut credentials[1..]);
        
        WithdrawalCredentials::from(credentials)
    }
}
```

### Key Management System

```rust
use keystore::{
    Keystore, PlainText, Cipher, Pbkdf2, Scrypt, Checksum, Kdf, 
    ChecksumModule, CipherModule, KdfModule
};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::path::PathBuf;

#[derive(ZeroizeOnDrop)]
pub struct KeyManager {
    keystores: HashMap<ValidatorIndex, SecureKeystore>,
    master_key: MasterKey,
    keystore_cache: LruCache<ValidatorIndex, Arc<SecretKey>>,
}

#[derive(ZeroizeOnDrop)]
struct SecureKeystore {
    keystore: Keystore,
    cached_key: Option<Arc<SecretKey>>,
    last_access: Instant,
}

#[derive(ZeroizeOnDrop)]
struct MasterKey {
    key: [u8; 32],
}

impl KeyManager {
    pub fn new(master_password: &str) -> Result<Self, KeyManagementError> {
        let master_key = Self::derive_master_key(master_password)?;
        
        Ok(Self {
            keystores: HashMap::new(),
            master_key,
            keystore_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        })
    }
    
    pub fn import_keystore(
        &mut self,
        keystore_path: &PathBuf,
        password: &str,
        validator_index: ValidatorIndex,
    ) -> Result<(), KeyManagementError> {
        // Load keystore from file
        let keystore_json = std::fs::read_to_string(keystore_path)?;
        let keystore: Keystore = serde_json::from_str(&keystore_json)?;
        
        // Verify password
        let secret_key = self.decrypt_keystore(&keystore, password)?;
        
        // Encrypt with master key for storage
        let encrypted_keystore = self.encrypt_with_master_key(&keystore)?;
        
        let secure_keystore = SecureKeystore {
            keystore: encrypted_keystore,
            cached_key: Some(Arc::new(secret_key)),
            last_access: Instant::now(),
        };
        
        self.keystores.insert(validator_index, secure_keystore);
        
        tracing::info!(
            validator_index = %validator_index,
            "Imported validator keystore"
        );
        
        Ok(())
    }
    
    pub fn get_signing_key(
        &mut self,
        validator_index: ValidatorIndex,
    ) -> Result<Arc<SecretKey>, KeyManagementError> {
        // Check cache first
        if let Some(key) = self.keystore_cache.get(&validator_index) {
            return Ok(key.clone());
        }
        
        // Load from secure storage
        let secure_keystore = self.keystores.get_mut(&validator_index)
            .ok_or(KeyManagementError::KeystoreNotFound(validator_index))?;
        
        if let Some(cached_key) = &secure_keystore.cached_key {
            let key = cached_key.clone();
            self.keystore_cache.put(validator_index, key.clone());
            secure_keystore.last_access = Instant::now();
            return Ok(key);
        }
        
        // Decrypt keystore
        let decrypted_keystore = self.decrypt_with_master_key(&secure_keystore.keystore)?;
        let secret_key = self.extract_secret_key(&decrypted_keystore)?;
        
        let key = Arc::new(secret_key);
        secure_keystore.cached_key = Some(key.clone());
        secure_keystore.last_access = Instant::now();
        self.keystore_cache.put(validator_index, key.clone());
        
        Ok(key)
    }
    
    pub fn sign_block(
        &mut self,
        block: &BeaconBlock,
        validator_index: ValidatorIndex,
        domain: Domain,
    ) -> Result<Signature, KeyManagementError> {
        let signing_key = self.get_signing_key(validator_index)?;
        let signing_root = compute_signing_root(block, domain)?;
        
        let signature = signing_key.sign(&signing_root);
        
        tracing::debug!(
            validator_index = %validator_index,
            slot = %block.slot,
            "Signed block"
        );
        
        Ok(signature)
    }
    
    pub fn sign_attestation(
        &mut self,
        attestation_data: &AttestationData,
        validator_index: ValidatorIndex,
        domain: Domain,
    ) -> Result<Signature, KeyManagementError> {
        let signing_key = self.get_signing_key(validator_index)?;
        let signing_root = compute_signing_root(attestation_data, domain)?;
        
        let signature = signing_key.sign(&signing_root);
        
        tracing::debug!(
            validator_index = %validator_index,
            slot = %attestation_data.slot,
            "Signed attestation"
        );
        
        Ok(signature)
    }
    
    fn derive_master_key(password: &str) -> Result<MasterKey, KeyManagementError> {
        let salt = b"panro_master_key_salt_v1";
        let mut key = [0u8; 32];
        
        pbkdf2::pbkdf2::<Hmac<Sha256>>(
            password.as_bytes(),
            salt,
            100_000, // iterations
            &mut key,
        );
        
        Ok(MasterKey { key })
    }
    
    fn encrypt_with_master_key(&self, keystore: &Keystore) -> Result<Keystore, KeyManagementError> {
        // Implementation would encrypt keystore with master key
        // For brevity, returning clone here
        Ok(keystore.clone())
    }
    
    fn decrypt_with_master_key(&self, keystore: &Keystore) -> Result<Keystore, KeyManagementError> {
        // Implementation would decrypt keystore with master key
        // For brevity, returning clone here
        Ok(keystore.clone())
    }
    
    pub fn periodic_cleanup(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        
        self.keystores.retain(|_, keystore| {
            if keystore.last_access < cutoff {
                // Clear cached key for security
                keystore.cached_key = None;
            }
            true
        });
        
        // Clear cache entries older than 30 minutes
        let cache_cutoff = Instant::now() - Duration::from_secs(1800);
        self.keystore_cache.clear();
    }
}

// Hardware Security Module integration
#[cfg(feature = "hsm")]
pub struct HsmKeyManager {
    hsm_session: HsmSession,
    key_handles: HashMap<ValidatorIndex, KeyHandle>,
}

#[cfg(feature = "hsm")]
impl HsmKeyManager {
    pub fn new(hsm_config: &HsmConfig) -> Result<Self, HsmError> {
        let hsm_session = HsmSession::connect(hsm_config)?;
        
        Ok(Self {
            hsm_session,
            key_handles: HashMap::new(),
        })
    }
    
    pub fn import_key_to_hsm(
        &mut self,
        secret_key: &SecretKey,
        validator_index: ValidatorIndex,
    ) -> Result<(), HsmError> {
        let key_handle = self.hsm_session.import_key(secret_key.to_bytes())?;
        self.key_handles.insert(validator_index, key_handle);
        Ok(())
    }
    
    pub fn sign_with_hsm(
        &self,
        message: &[u8],
        validator_index: ValidatorIndex,
    ) -> Result<Signature, HsmError> {
        let key_handle = self.key_handles.get(&validator_index)
            .ok_or(HsmError::KeyNotFound(validator_index))?;
        
        let signature_bytes = self.hsm_session.sign(key_handle, message)?;
        let signature = Signature::from_bytes(&signature_bytes)?;
        
        Ok(signature)
    }
}
```

## Network Security

### TLS/Noise Protocol Implementation

```rust
use libp2p::noise::{Keypair, NoiseConfig, X25519Spec, XX};
use libp2p::core::transport::Boxed;
use libp2p::core::upgrade;
use libp2p::tcp::TcpConfig;
use libp2p::Transport;

pub struct SecureTransport {
    noise_keypair: Keypair<X25519Spec>,
    transport: Boxed<(PeerId, StreamMuxerBox)>,
}

impl SecureTransport {
    pub fn new() -> Result<Self, TransportError> {
        // Generate or load noise keypair
        let noise_keypair = Keypair::<X25519Spec>::new()
            .into_authentic(&local_keypair)?;
        
        // Create transport with Noise encryption
        let transport = TcpConfig::new()
            .nodelay(true)
            .upgrade(upgrade::Version::V1)
            .authenticate(NoiseConfig::xx(noise_keypair.clone()).into_authenticated())
            .multiplex(yamux::YamuxConfig::default())
            .timeout(Duration::from_secs(20))
            .boxed();
        
        Ok(Self {
            noise_keypair,
            transport,
        })
    }
    
    pub fn with_tls_config(mut self, tls_config: TlsConfig) -> Self {
        // Add TLS layer for additional security
        self
    }
}

// Rate limiting and DDoS protection
pub struct NetworkSecurityManager {
    rate_limiters: HashMap<PeerId, TokenBucket>,
    connection_tracker: ConnectionTracker,
    reputation_system: PeerReputation,
    firewall: NetworkFirewall,
}

impl NetworkSecurityManager {
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            rate_limiters: HashMap::new(),
            connection_tracker: ConnectionTracker::new(config.max_connections_per_ip),
            reputation_system: PeerReputation::new(),
            firewall: NetworkFirewall::new(config),
        }
    }
    
    pub async fn validate_incoming_connection(
        &mut self,
        peer_id: PeerId,
        remote_addr: SocketAddr,
    ) -> Result<ConnectionDecision, SecurityError> {
        // Check firewall rules
        if !self.firewall.is_allowed(&remote_addr) {
            return Ok(ConnectionDecision::Reject("Blocked by firewall".to_string()));
        }
        
        // Check connection limits
        if !self.connection_tracker.can_accept_connection(&remote_addr) {
            return Ok(ConnectionDecision::Reject("Connection limit exceeded".to_string()));
        }
        
        // Check peer reputation
        let reputation = self.reputation_system.get_reputation(peer_id).await;
        if reputation.is_blacklisted() {
            return Ok(ConnectionDecision::Reject("Peer blacklisted".to_string()));
        }
        
        // Apply rate limiting
        let rate_limiter = self.rate_limiters
            .entry(peer_id)
            .or_insert_with(|| TokenBucket::new(10, 100)); // 10/sec, burst 100
        
        if !rate_limiter.try_consume(1) {
            return Ok(ConnectionDecision::RateLimit);
        }
        
        Ok(ConnectionDecision::Accept)
    }
    
    pub async fn handle_message_security(
        &mut self,
        peer_id: PeerId,
        message: &NetworkMessage,
    ) -> Result<MessageSecurityResult, SecurityError> {
        // Message size validation
        if message.encoded_len() > MAX_MESSAGE_SIZE {
            self.reputation_system.report_violation(
                peer_id,
                SecurityViolation::OversizedMessage(message.encoded_len())
            ).await;
            return Ok(MessageSecurityResult::Reject("Message too large".to_string()));
        }
        
        // Message frequency validation
        if !self.check_message_frequency(peer_id, message).await? {
            return Ok(MessageSecurityResult::RateLimit);
        }
        
        // Content validation
        if let Err(violation) = self.validate_message_content(message).await {
            self.reputation_system.report_violation(peer_id, violation).await;
            return Ok(MessageSecurityResult::Reject("Invalid message content".to_string()));
        }
        
        Ok(MessageSecurityResult::Accept)
    }
    
    async fn validate_message_content(
        &self,
        message: &NetworkMessage,
    ) -> Result<(), SecurityViolation> {
        match message {
            NetworkMessage::BeaconBlocks { blocks } => {
                for block in blocks {
                    self.validate_block_security(block).await?;
                }
            }
            NetworkMessage::Attestation { attestation } => {
                self.validate_attestation_security(attestation).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn validate_block_security(
        &self,
        block: &SignedBeaconBlock,
    ) -> Result<(), SecurityViolation> {
        // Check block slot bounds
        let current_slot = self.get_current_slot().await;
        let block_slot = block.message().slot();
        
        if block_slot > current_slot + FUTURE_SLOT_TOLERANCE {
            return Err(SecurityViolation::FutureBlock(block_slot));
        }
        
        if block_slot < current_slot.saturating_sub(PAST_SLOT_TOLERANCE) {
            return Err(SecurityViolation::StaleBlock(block_slot));
        }
        
        Ok(())
    }
}

// Peer reputation system
pub struct PeerReputation {
    scores: HashMap<PeerId, ReputationScore>,
    violations: HashMap<PeerId, Vec<SecurityViolation>>,
    blacklist: HashSet<PeerId>,
}

#[derive(Debug, Clone)]
pub struct ReputationScore {
    score: f64,
    last_updated: Instant,
    connection_time: Instant,
    successful_interactions: u64,
    failed_interactions: u64,
}

impl PeerReputation {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            violations: HashMap::new(),
            blacklist: HashSet::new(),
        }
    }
    
    pub async fn report_violation(
        &mut self,
        peer_id: PeerId,
        violation: SecurityViolation,
    ) {
        let violations = self.violations.entry(peer_id).or_default();
        violations.push(violation.clone());
        
        // Update reputation score
        let score = self.scores.entry(peer_id).or_insert_with(|| ReputationScore {
            score: 100.0,
            last_updated: Instant::now(),
            connection_time: Instant::now(),
            successful_interactions: 0,
            failed_interactions: 0,
        });
        
        let penalty = match violation {
            SecurityViolation::OversizedMessage(_) => 5.0,
            SecurityViolation::InvalidSignature => 20.0,
            SecurityViolation::FutureBlock(_) => 10.0,
            SecurityViolation::StaleBlock(_) => 2.0,
            SecurityViolation::SpamDetected => 15.0,
            SecurityViolation::MalformedMessage => 8.0,
        };
        
        score.score = (score.score - penalty).max(0.0);
        score.failed_interactions += 1;
        score.last_updated = Instant::now();
        
        // Blacklist if score too low
        if score.score < 10.0 {
            self.blacklist.insert(peer_id);
            tracing::warn!(
                peer_id = %peer_id,
                score = score.score,
                "Blacklisted peer due to low reputation"
            );
        }
        
        tracing::debug!(
            peer_id = %peer_id,
            violation = ?violation,
            new_score = score.score,
            "Reported security violation"
        );
    }
    
    pub async fn report_success(&mut self, peer_id: PeerId) {
        let score = self.scores.entry(peer_id).or_insert_with(|| ReputationScore {
            score: 100.0,
            last_updated: Instant::now(),
            connection_time: Instant::now(),
            successful_interactions: 0,
            failed_interactions: 0,
        });
        
        score.score = (score.score + 0.1).min(100.0);
        score.successful_interactions += 1;
        score.last_updated = Instant::now();
    }
    
    pub async fn get_reputation(&self, peer_id: PeerId) -> &ReputationScore {
        static DEFAULT_SCORE: ReputationScore = ReputationScore {
            score: 50.0,
            last_updated: Instant::now(),
            connection_time: Instant::now(),
            successful_interactions: 0,
            failed_interactions: 0,
        };
        
        self.scores.get(&peer_id).unwrap_or(&DEFAULT_SCORE)
    }
    
    pub fn is_blacklisted(&self, peer_id: PeerId) -> bool {
        self.blacklist.contains(&peer_id)
    }
}
```

## Input Validation and Sanitization

### Message Validation Framework

```rust
use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

pub struct MessageValidator {
    chain_config: ChainConfig,
    signature_verifier: Arc<BlsSignatureVerifier>,
    spam_detector: SpamDetector,
}

impl MessageValidator {
    pub async fn validate_beacon_block(
        &self,
        block: &SignedBeaconBlock,
        current_state: &BeaconState,
    ) -> Result<ValidationResult, ValidationError> {
        // Structural validation
        self.validate_block_structure(block)?;
        
        // State transition validation
        self.validate_block_state_transition(block, current_state).await?;
        
        // Signature validation
        self.validate_block_signatures(block, current_state).await?;
        
        // Spam detection
        if self.spam_detector.is_spam_block(block).await? {
            return Ok(ValidationResult::Ignore("Detected spam".to_string()));
        }
        
        Ok(ValidationResult::Accept)
    }
    
    fn validate_block_structure(&self, block: &SignedBeaconBlock) -> Result<(), ValidationError> {
        let message = block.message();
        
        // Validate slot bounds
        if message.slot() == 0 {
            return Err(ValidationError::InvalidSlot("Genesis slot not allowed".to_string()));
        }
        
        // Validate proposer index
        if message.proposer_index() >= self.chain_config.validator_registry_limit {
            return Err(ValidationError::InvalidProposer(message.proposer_index()));
        }
        
        // Validate body structure
        self.validate_block_body(message.body())?;
        
        Ok(())
    }
    
    fn validate_block_body(&self, body: &BeaconBlockBody) -> Result<(), ValidationError> {
        // Validate attestations count
        if body.attestations().len() > MAX_ATTESTATIONS {
            return Err(ValidationError::TooManyAttestations(body.attestations().len()));
        }
        
        // Validate deposits count
        if body.deposits().len() > MAX_DEPOSITS {
            return Err(ValidationError::TooManyDeposits(body.deposits().len()));
        }
        
        // Validate voluntary exits count
        if body.voluntary_exits().len() > MAX_VOLUNTARY_EXITS {
            return Err(ValidationError::TooManyVoluntaryExits(body.voluntary_exits().len()));
        }
        
        // Validate proposer slashings count
        if body.proposer_slashings().len() > MAX_PROPOSER_SLASHINGS {
            return Err(ValidationError::TooManyProposerSlashings(body.proposer_slashings().len()));
        }
        
        // Validate attester slashings count
        if body.attester_slashings().len() > MAX_ATTESTER_SLASHINGS {
            return Err(ValidationError::TooManyAttesterSlashings(body.attester_slashings().len()));
        }
        
        // Validate individual components
        for attestation in body.attestations() {
            self.validate_attestation_structure(attestation)?;
        }
        
        for deposit in body.deposits() {
            self.validate_deposit_structure(deposit)?;
        }
        
        Ok(())
    }
    
    fn validate_attestation_structure(&self, attestation: &Attestation) -> Result<(), ValidationError> {
        let data = &attestation.data;
        
        // Validate committee index
        if data.index >= MAX_COMMITTEES_PER_SLOT {
            return Err(ValidationError::InvalidCommitteeIndex(data.index));
        }
        
        // Validate epoch bounds
        if data.target.epoch < data.source.epoch {
            return Err(ValidationError::InvalidEpochProgression {
                source: data.source.epoch,
                target: data.target.epoch,
            });
        }
        
        // Validate slot bounds
        let slot_epoch = data.slot.epoch(SLOTS_PER_EPOCH);
        if slot_epoch != data.target.epoch {
            return Err(ValidationError::MismatchedSlotEpoch {
                slot_epoch,
                target_epoch: data.target.epoch,
            });
        }
        
        // Validate aggregation bits
        if attestation.aggregation_bits.len() == 0 {
            return Err(ValidationError::EmptyAggregationBits);
        }
        
        if attestation.aggregation_bits.len() > MAX_VALIDATORS_PER_COMMITTEE {
            return Err(ValidationError::OversizedAggregationBits(attestation.aggregation_bits.len()));
        }
        
        Ok(())
    }
    
    pub async fn validate_network_message(
        &self,
        peer_id: PeerId,
        message: &NetworkMessage,
    ) -> Result<ValidationResult, ValidationError> {
        // Basic size validation
        let encoded_size = message.encoded_len();
        if encoded_size > MAX_NETWORK_MESSAGE_SIZE {
            return Err(ValidationError::MessageTooLarge {
                size: encoded_size,
                max_size: MAX_NETWORK_MESSAGE_SIZE,
            });
        }
        
        // Rate limiting validation
        if !self.check_message_rate_limit(peer_id, message).await? {
            return Ok(ValidationResult::RateLimit);
        }
        
        // Message-specific validation
        match message {
            NetworkMessage::BeaconBlocks { blocks } => {
                for block in blocks {
                    self.validate_gossip_block(block).await?;
                }
            }
            NetworkMessage::Attestation { attestation } => {
                self.validate_gossip_attestation(attestation).await?;
            }
            NetworkMessage::AggregateAndProof { aggregate } => {
                self.validate_aggregate_and_proof(aggregate).await?;
            }
            _ => {}
        }
        
        Ok(ValidationResult::Accept)
    }
    
    async fn validate_gossip_block(&self, block: &SignedBeaconBlock) -> Result<(), ValidationError> {
        // Check if block is from the correct slot range
        let current_slot = self.get_current_slot().await;
        let block_slot = block.message().slot();
        
        if block_slot > current_slot {
            return Err(ValidationError::FutureBlock(block_slot));
        }
        
        if block_slot < current_slot.saturating_sub(GOSSIP_CLOCK_DISPARITY) {
            return Err(ValidationError::StaleBlock(block_slot));
        }
        
        // Additional gossip-specific validations
        self.validate_block_structure(block)?;
        
        Ok(())
    }
}

// Spam detection system
pub struct SpamDetector {
    message_history: LruCache<MessageHash, Instant>,
    peer_frequencies: HashMap<PeerId, FrequencyTracker>,
    content_filters: Vec<Box<dyn ContentFilter>>,
}

impl SpamDetector {
    pub async fn is_spam_block(&mut self, block: &SignedBeaconBlock) -> Result<bool, SpamError> {
        let block_hash = block.canonical_root();
        
        // Check for duplicate blocks
        if self.message_history.contains(&block_hash) {
            return Ok(true);
        }
        
        // Check block frequency patterns
        if self.is_excessive_block_frequency(block).await? {
            return Ok(true);
        }
        
        // Content-based spam detection
        for filter in &self.content_filters {
            if filter.is_spam_content(block).await? {
                return Ok(true);
            }
        }
        
        // Record message
        self.message_history.put(block_hash, Instant::now());
        
        Ok(false)
    }
    
    async fn is_excessive_block_frequency(&self, block: &SignedBeaconBlock) -> Result<bool, SpamError> {
        // Check if proposer is submitting too many blocks
        let proposer_index = block.message().proposer_index();
        let slot = block.message().slot();
        
        // More than one block per slot from same proposer is spam
        // (This is a simplified check; in practice, multiple blocks per slot 
        // from the same proposer should be investigated for slashing)
        
        Ok(false) // Placeholder implementation
    }
}
```

## Access Control and Authorization

### Role-Based Access Control

```rust
use jsonwebtoken::{decode, encode, Header, Validation, DecodingKey, EncodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,          // Subject (user ID)
    pub role: UserRole,       // User role
    pub permissions: Vec<Permission>,  // Specific permissions
    pub exp: usize,          // Expiration time
    pub iat: usize,          // Issued at
    pub iss: String,         // Issuer
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Operator,
    Validator,
    Observer,
    Readonly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    // Node management
    RestartNode,
    UpdateConfig,
    ViewConfig,
    ViewLogs,
    
    // Validator operations
    ImportKeys,
    SignBlocks,
    SignAttestations,
    ViewValidatorStatus,
    
    // Network operations
    ManagePeers,
    ViewNetworkStatus,
    
    // Database operations
    BackupDatabase,
    RestoreDatabase,
    ViewDatabaseStats,
    
    // Monitoring
    ViewMetrics,
    ConfigureAlerts,
    
    // API access
    BeaconAPI,
    ValidatorAPI,
    NodeAPI,
    DebugAPI,
}

pub struct AccessController {
    jwt_secret: Vec<u8>,
    role_permissions: HashMap<UserRole, Vec<Permission>>,
    api_keys: HashMap<String, ApiKeyInfo>,
}

#[derive(Debug, Clone)]
struct ApiKeyInfo {
    role: UserRole,
    permissions: Vec<Permission>,
    created_at: SystemTime,
    last_used: SystemTime,
    rate_limit: RateLimit,
}

impl AccessController {
    pub fn new(jwt_secret: &[u8]) -> Self {
        let mut role_permissions = HashMap::new();
        
        // Admin permissions
        role_permissions.insert(UserRole::Admin, vec![
            Permission::RestartNode,
            Permission::UpdateConfig,
            Permission::ViewConfig,
            Permission::ViewLogs,
            Permission::ImportKeys,
            Permission::SignBlocks,
            Permission::SignAttestations,
            Permission::ViewValidatorStatus,
            Permission::ManagePeers,
            Permission::ViewNetworkStatus,
            Permission::BackupDatabase,
            Permission::RestoreDatabase,
            Permission::ViewDatabaseStats,
            Permission::ViewMetrics,
            Permission::ConfigureAlerts,
            Permission::BeaconAPI,
            Permission::ValidatorAPI,
            Permission::NodeAPI,
            Permission::DebugAPI,
        ]);
        
        // Operator permissions
        role_permissions.insert(UserRole::Operator, vec![
            Permission::ViewConfig,
            Permission::ViewLogs,
            Permission::ViewValidatorStatus,
            Permission::ViewNetworkStatus,
            Permission::ViewDatabaseStats,
            Permission::ViewMetrics,
            Permission::BeaconAPI,
            Permission::ValidatorAPI,
            Permission::NodeAPI,
        ]);
        
        // Validator permissions
        role_permissions.insert(UserRole::Validator, vec![
            Permission::SignBlocks,
            Permission::SignAttestations,
            Permission::ViewValidatorStatus,
            Permission::ValidatorAPI,
        ]);
        
        // Observer permissions
        role_permissions.insert(UserRole::Observer, vec![
            Permission::ViewNetworkStatus,
            Permission::ViewMetrics,
            Permission::BeaconAPI,
        ]);
        
        // Readonly permissions
        role_permissions.insert(UserRole::Readonly, vec![
            Permission::ViewMetrics,
            Permission::BeaconAPI,
        ]);
        
        Self {
            jwt_secret: jwt_secret.to_vec(),
            role_permissions,
            api_keys: HashMap::new(),
        }
    }
    
    pub fn generate_jwt_token(
        &self,
        user_id: &str,
        role: UserRole,
        duration: Duration,
    ) -> Result<String, AuthError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        
        let permissions = self.role_permissions
            .get(&role)
            .cloned()
            .unwrap_or_default();
        
        let claims = Claims {
            sub: user_id.to_string(),
            role,
            permissions,
            exp: now + duration.as_secs() as usize,
            iat: now,
            iss: "panro".to_string(),
        };
        
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(&self.jwt_secret);
        
        encode(&header, &claims, &encoding_key)
            .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }
    
    pub fn validate_jwt_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        let decoding_key = DecodingKey::from_secret(&self.jwt_secret);
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        
        Ok(token_data.claims)
    }
    
    pub fn check_permission(
        &self,
        claims: &Claims,
        required_permission: Permission,
    ) -> Result<bool, AuthError> {
        // Check if user has the specific permission
        if claims.permissions.contains(&required_permission) {
            return Ok(true);
        }
        
        // Check if user's role has the permission
        if let Some(role_permissions) = self.role_permissions.get(&claims.role) {
            if role_permissions.contains(&required_permission) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    pub fn generate_api_key(
        &mut self,
        role: UserRole,
        custom_permissions: Option<Vec<Permission>>,
    ) -> Result<String, AuthError> {
        let api_key = Self::generate_secure_key();
        
        let permissions = custom_permissions.unwrap_or_else(|| {
            self.role_permissions.get(&role).cloned().unwrap_or_default()
        });
        
        let api_key_info = ApiKeyInfo {
            role,
            permissions,
            created_at: SystemTime::now(),
            last_used: SystemTime::now(),
            rate_limit: RateLimit::new(1000, Duration::from_secs(3600)), // 1000/hour
        };
        
        self.api_keys.insert(api_key.clone(), api_key_info);
        
        Ok(api_key)
    }
    
    pub fn validate_api_key(&mut self, api_key: &str) -> Result<&ApiKeyInfo, AuthError> {
        let api_key_info = self.api_keys.get_mut(api_key)
            .ok_or(AuthError::InvalidApiKey)?;
        
        // Check rate limit
        if !api_key_info.rate_limit.check() {
            return Err(AuthError::RateLimitExceeded);
        }
        
        api_key_info.last_used = SystemTime::now();
        
        Ok(api_key_info)
    }
    
    fn generate_secure_key() -> String {
        use rand::{thread_rng, Rng};
        use rand::distributions::Alphanumeric;
        
        let key: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        
        format!("panro_{}", key)
    }
}

// HTTP middleware for authentication
pub struct AuthMiddleware {
    access_controller: Arc<AccessController>,
}

impl AuthMiddleware {
    pub fn new(access_controller: Arc<AccessController>) -> Self {
        Self { access_controller }
    }
    
    pub async fn authenticate_request(
        &self,
        req: &HttpRequest,
    ) -> Result<Claims, AuthError> {
        // Try JWT token first
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return self.access_controller.validate_jwt_token(token);
                }
            }
        }
        
        // Try API key
        if let Some(api_key) = req.headers().get("X-API-Key") {
            if let Ok(key_str) = api_key.to_str() {
                let api_key_info = self.access_controller.validate_api_key(key_str)?;
                
                return Ok(Claims {
                    sub: "api_key_user".to_string(),
                    role: api_key_info.role.clone(),
                    permissions: api_key_info.permissions.clone(),
                    exp: (SystemTime::now() + Duration::from_secs(3600))
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as usize,
                    iat: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as usize,
                    iss: "panro".to_string(),
                });
            }
        }
        
        Err(AuthError::NoAuthProvided)
    }
    
    pub fn authorize_endpoint(
        &self,
        claims: &Claims,
        endpoint: &str,
    ) -> Result<bool, AuthError> {
        let required_permission = match endpoint {
            "/eth/v1/beacon/blocks" => Permission::BeaconAPI,
            "/eth/v1/validator/duties" => Permission::ValidatorAPI,
            "/eth/v1/node/health" => Permission::BeaconAPI,
            "/api/v1/admin/restart" => Permission::RestartNode,
            "/api/v1/admin/config" => Permission::ViewConfig,
            "/api/v1/keys/import" => Permission::ImportKeys,
            "/metrics" => Permission::ViewMetrics,
            _ => return Ok(false), // Default deny
        };
        
        self.access_controller.check_permission(claims, required_permission)
    }
}
```

## Security Configuration

### Security Hardening Configuration

```toml
# security.toml
[security]
enabled = true
strict_mode = true

# Authentication
[security.auth]
jwt_enabled = true
jwt_secret_file = "/etc/panro/jwt.secret"
jwt_expiry_hours = 24
api_keys_enabled = true
session_timeout_minutes = 30

# TLS/SSL
[security.tls]
enabled = true
cert_file = "/etc/panro/certs/server.crt"
key_file = "/etc/panro/certs/server.key"
ca_file = "/etc/panro/certs/ca.crt"
min_version = "1.3"
cipher_suites = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256",
    "TLS_AES_128_GCM_SHA256"
]

# Network security
[security.network]
firewall_enabled = true
ddos_protection = true
rate_limiting = true
max_connections_per_ip = 10
connection_timeout_seconds = 30

# Rate limiting configuration
[security.rate_limiting]
global_requests_per_second = 1000
per_ip_requests_per_second = 10
per_endpoint_requests_per_second = 100
burst_allowance = 50

# Input validation
[security.validation]
strict_validation = true
max_message_size_bytes = 1048576  # 1MB
max_attestations_per_block = 128
max_deposits_per_block = 16
sanitize_inputs = true

# Key management
[security.keys]
key_derivation_iterations = 100000
secure_memory = true
key_rotation_enabled = true
key_rotation_interval_days = 90
hsm_enabled = false
hsm_config_file = "/etc/panro/hsm.conf"

# Logging and monitoring
[security.monitoring]
security_event_logging = true
failed_auth_threshold = 5
suspicious_activity_detection = true
intrusion_detection = true
audit_trail = true

# Backup encryption
[security.backup]
encryption_enabled = true
encryption_algorithm = "AES-256-GCM"
key_file = "/etc/panro/backup.key"
verify_integrity = true

# Update security
[security.updates]
auto_security_updates = false
signature_verification = true
update_channel = "stable"
```
