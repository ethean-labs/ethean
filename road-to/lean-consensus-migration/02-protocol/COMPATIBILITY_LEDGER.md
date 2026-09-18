# Compatibility Ledger

## Purpose

The compatibility ledger is the single machine-readable input for protocol selection, startup compatibility checks, fixture provenance, and release evidence.

Phase 00 (2026-09-19) froze `leanSpec@0b7d33ec` and the production fixture digest. Remaining hard gates: **leanMetrics pin (OSD-009)** and **Ethean Rust toolchain (TBD)**. Secondary network identity / discovery / signer fields stay open for later phases.

Authoritative lock: [`../../../spec/pins/phase-00.lock.toml`](../../../spec/pins/phase-00.lock.toml).

## Canonical machine-readable fields

```yaml
ledger_schema: ethean-lean-compatibility-v1
recorded_at_utc: "2026-09-19T00:00:00Z"
profile_status: phase_00_locked_with_open_gates
profile_status_reason: >-
  leanSpec commit, lstar constants, XMSS PROD_CONFIG, Type-2 block proof,
  aggregation tag, Snappy surfaces, and fixture digest are resolved.
  leanMetrics (OSD-009) and Ethean rustc/toolchain remain unresolved.
  Compatibility fingerprint stays unavailable until remaining required
  selected fields are concrete.

authority:
  lean_spec:
    status: resolved
    repository: https://github.com/leanEthereum/leanSpec
    tracking_branch: main
    selected_commit: 0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8
    selected_tree: 4a0a358abf0447c26ad64ad6e57fd1e59ecccafd
    fork: lstar
    fork_name: LstarSpec
    commit_signature: verified_on_github
    also_current_main_at_retrieval: true
    observed_commits:
      pq_devnet_4_note: 0c9528ac6f403f913caf6d9c450879c36d361742
      zeam_gitlink: 8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54
      lantern_gitlink: 8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54
      gean_fixture_source: eca701efeb5931010fe63925cd203c9ee55b2dbc
      qlean_aggregate_bound_citation: 6430aaf7cb505ec76bb1af6045ffcd3b41899e42
    resolution_gate: phase-00.lock.toml
  pq_devnet:
    status: unresolved
    selected_generation: null
    selected_plan_commit: null
    selected_quickstart_commit: null
    observed_quickstart_commits:
      zeam: f78bcb3b097dbba723f805da5511f9a0313354b6
      lantern: 7c7c72d2d815e93f59cfcf849b6d3ffaf3a8873
  fixtures:
    status: resolved
    release_identity: "Latest production fixtures"
    release_tag_locator: latest
    asset_url: https://github.com/leanEthereum/leanSpec/releases/download/latest/fixtures-prod-scheme.tar.gz
    asset_name: fixtures-prod-scheme.tar.gz
    asset_size_bytes: 154123448
    asset_sha256: 21d9de7056b4e658031dc09e50c0e9dc1b0206089253258ce69ecff01154d4bd
    generator_commit: 0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8
    target_commitish: 0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8
    generation_command: null
    crypto_scheme: prod
    local_modifications: forbidden
    required_hash_algorithm: sha256
    manifest: spec/fixtures/phase-00/manifest.toml
    tarball_in_git: false

ethean_planning:
  planning_head: f82bbbb179f04e742a2321fe4d50a76eb46c36be
  note: Separate from protocol pin.

crypto:
  status: resolved
  signature_repository: leanSpec_internal_xmss
  signature_commit: 0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8
  aggregation_repository: https://github.com/anshalshukla/leanMultisig-py
  aggregation_tag: v0.0.9
  aggregation_annotated_tag_object: 6f79724eed9a4b33690cb6d52cbe60fb455d8770
  aggregation_commit: 39b9c397fd4d2c358f8efe2815d4fb78f0e47e67
  construction: leanSpec_internal_xmss
  field: koalabear
  algebraic_hash: poseidon
  hash_instance: poseidon_koalabear_prod_config
  dimension: 46
  winternitz_base: 8
  lifetime_log2: 32
  activation_window: null
  slot_to_signature_index_rule: null
  log_inv_rate: null
  public_key_bytes: 52
  signature_bytes: 2536
  proof_max_bytes: null
  secret_key_serialization: null
  type1_container: not_selected
  type2_container: MultiMessageAggregate
  block_component_order: attestations_then_proposer_in_block_proof
  observed_generations:
    standalone_leansig_dim46:
      construction: generalized_xmss_aborting_target_sum
      hash: poseidon1
      dimension: 46
      winternitz_base: 8
      lifetime_log2: 32
      public_key_bytes: 52
      signature_bytes: 2536
      observed_signature_commit: 15cbdd43ec8525aa43fea2f42cafc5ed366084ae
      observed_type2_commit: e2592df4e30fdddbbf8ae26a333116c68cec7026
      observed_in: [ream, zeam, ethlambda, lantern]
    leanvm_internal_v42:
      construction: leanvm_internal_xmss
      field: koalabear
      hash: poseidon2
      dimension: 42
      winternitz_base: 8
      lifetime_log2: 32
      public_key_bytes: 32
      signature_bytes: 1208
      observed_leanvm_commit: a5909d18647de6aed38640c098d9177fab2bf36a
      observed_in: [gean]
    historical_peam:
      construction: generalized_xmss
      dimension: 64
      winternitz_base: 8
      lifetime_log2: 32
      public_key_bytes: 52
      signature_bytes: 3112
      observed_signature_commit: 73bedc26ed961b110df7ac2e234dc11361a4bf25
      observed_aggregation_commit: e4474138487eeb1ed7c2e1013674fe80ac9f3165
      protocol_generation: devnet_2
  observed_proof_limits_bytes:
    ream: 524288
    zeam: 524288
    ethlambda: 524288
    lantern: 524288
    gean: 524288
    qlean_mini: 1048576
    peam: 1048576
  observed_log_inv_rate:
    ream: 2
    qlean_mini: 2
    ethlambda: 2
    gean: 2
    pq_devnet_4_allowed_range: [1, 4]

consensus:
  status: resolved
  max_attestation_data: 8
  max_attestations_data_name: MAX_ATTESTATIONS_DATA
  max_validators: 4096
  historical_roots_limit: 262144
  slot_duration_milliseconds: 4000
  intervals_per_slot: 5
  proposer_selection: null
  subnet_assignment: null
  fork_choice: lstar_modified_3sf_era
  finality: lstar_modified_3sf_mini
  vote_promotion_boundary: null
  safe_target_rule: null
  tie_break_rule: null
  observed_max_attestation_data:
    devnet5_shaped_clients: 8
    qlean_mini: 16
    pq_devnet_4_objectives: 16
    pq_devnet_4_summary: 8
  observed_finality:
    peer_snapshots: modified_3sf_mini
    roadmap_direction: pq_heartbeat_goldfish
    selected: modified_3sf_era_lstar

network:
  status: partially_resolved
  network_name: null
  fork_identifier_bytes: null
  fork_identifier_derivation: null
  topic_template: /leanconsensus/{fork}/{message}/ssz_snappy
  block_topic: null
  attestation_topic: null
  aggregation_topic: null
  gossip_compression: raw_snappy
  gossip_message_id_algorithm: null
  gossip_message_id_preimage: null
  gossip_message_id_length_bytes: null
  gossip_protocol_profile: null
  max_gossip_decoded_bytes: null
  transport: null
  discovery: null
  enr_schema: null
  status_protocol: /leanconsensus/req/status/1/ssz_snappy
  blocks_by_root_protocol: /leanconsensus/req/blocks_by_root/1/ssz_snappy
  blocks_by_range_protocol: /leanconsensus/req/blocks_by_range/1/ssz_snappy
  request_compression: framed_snappy
  request_framing: snappy_framed_with_varint_length
  response_codes: null
  stream_termination: null
  max_blocks_per_request: null
  observed_common_protocol_ids:
    status: /leanconsensus/req/status/1/ssz_snappy
    blocks_by_root: /leanconsensus/req/blocks_by_root/1/ssz_snappy
    blocks_by_range: /leanconsensus/req/blocks_by_range/1/ssz_snappy
  observed_common_topic_template: /leanconsensus/{fork}/{message}/ssz_snappy
  observed_fork_identifiers:
    dummy: "12345678"
    ream_test_variant: "0x12345678"
    peam_default: devnet3
  observed_message_id_conflict:
    lantern_and_gean: domain_then_topic_length_then_topic_then_payload
    qlean_mini: topic_length_then_topic_then_domain_then_payload
  observed_compression:
    gossip: raw_snappy
    request_response: framed_snappy
  observed_max_decoded_bytes:
    ream: 10485760
    lantern: 10485760
    gean: 10485760
    ethlambda_approximate_transmit_cap: 12582912
  observed_discovery:
    ream_lean_runtime: static_bootnodes
    zeam: static_enrs
    qlean_mini: static_enrs
    ethlambda: optional_discv5
    lantern: static_enrs
    gean: static_enrs
    peam: mdns_and_static_enrs

checkpoint:
  status: unresolved
  trust_model: null
  trusted_root_source: null
  transport_authentication: null
  state_block_pair_required: null
  block_proof_verification_required: null
  canonical_membership_evidence: null
  rollback_floor: null
  observed_peer_behavior: >-
    Peers generally verify internal state/block/genesis consistency but trust
    an operator-selected HTTP endpoint and do not prove canonical membership.

signer:
  status: unresolved
  role_separated_keys: null
  durable_last_signed_record: null
  atomic_reservation_before_publish: null
  crash_recovery_rule: null
  rollback_detection: null
  concurrent_lease_rule: null
  key_exhaustion_rule: null
  backup_restore_rule: null
  observed_risk: >-
    No inspected peer establishes a complete durable duplicate-signing and
    rollback-safety contract for stateful XMSS at its audited commit.

toolchain:
  status: unresolved
  ethean_rust_version: TBD
  rust_toolchain_toml: required_phase_01_or_later
  cargo_lock_sha256: null
  rust_target: null
  build_image_digest: null
  fixture_python_version: null
  fixture_uv_version: null
  code_generator_versions: {}
  note: >-
    rustc missing on Phase 00 authoring host. Do not invent a version.
  observed_peer_toolchains:
    ream:
      rust: 1.98.0
      edition: "2024"
    zeam:
      zig: 0.16.0
      rust: 1.96.0
    qlean_mini:
      language: c++23
      cmake_minimum: 3.25
      observed_ci_gcc: 14
      observed_ci_python: "3.12"
      observed_ci_cmake: 3.31.1
      rust: unpinned_stable
    ethlambda:
      rust: 1.97.1
      edition: "2024"
    lantern:
      language: c17
      cmake_minimum: 3.20
      rust: unpinned_stable
    gean:
      go_mod: 1.25.7
      rust: 1.92.0
      fixture_rust: unpinned_nightly
    peam:
      edition: "2024"
      rust: unpinned_stable

observability:
  status: unresolved
  osd: OSD-009
  lean_metrics_commit: null
  lean_spec_stack: prometheus-client
  lean_spec_range: ">=0.21.0,<1"
  fallback_until_pin: ethean_namespace_with_schema_versioning

ssz:
  status: resolved
  python_package: eth-ssz-specs
  version_range: ">=0.1.0,<0.2"

peer_evidence:
  ream: b003b250f51c038cd5e16b8da02694ee0db1997e
  zeam: 6495beb6b1a584e41c12b3569d9a509abd906259
  qlean_mini: 55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0
  ethlambda: 313b22d4aa15174b87319002d813926ab7eb6411
  lantern: 440e34727ab3fb447de68d91a1e49be5327706e8
  gean: b78f6d737f4df57a72d5e230635681235fda8024
  peam: 6628e7a564098e592a49b9af0ad7b5dcda0a71fc

compatibility_fingerprint:
  status: unavailable_until_profile_fully_resolved
  algorithm: sha256
  canonicalization: rfc8785_json
  domain: ethean-lean-profile-v1
  digest_hex: null
```

## Required fixture asset integrity

Every fixture bundle used by P1–P4 must have:

1. the full `leanSpec` source commit;
2. immutable release identity and exact asset name;
3. final resolved HTTPS URL;
4. byte length;
5. lowercase SHA-256 calculated after download;
6. fixture generation command and exact Python/`uv` versions (still open for Phase 00 — digest pin is present);
7. selected production or test crypto scheme;
8. a statement that local modifications are absent;
9. an extracted manifest SHA-256 (after first verified extract in CI);
10. CI verification before extraction and before test discovery.

A GitHub release tag, `latest` URL, ETag, checksum obtained from mutable metadata, or archive filename alone is insufficient. Phase 00 pins **digest + size + generator commit**; the tag is only a download locator.

## Compatibility fingerprint

The fingerprint binds all consensus and wire inputs used by a binary.

1. Remove `recorded_at_utc`, `observed_*`, `peer_evidence`, explanatory status reasons, and the `compatibility_fingerprint` object.
2. Reject the record if any required selected field is `null`, `unresolved`, `unavailable`, branch-based, or tag-only.
3. Serialize the remaining selected profile as RFC 8785 canonical JSON.
4. Compute:

   `SHA256(UTF8("ethean-lean-profile-v1") || 0x00 || canonical_json_bytes)`

5. Encode the 32-byte result as 64 lowercase hexadecimal characters.

The digest must be embedded in the binary, emitted at startup, persisted with the database schema and signer state, included in interop logs, and published with release artifacts. It is a compatibility identity, not a substitute for a genesis root or fork digest.

**Phase 00:** fingerprint remains unavailable while OSD-009, Rust toolchain, and remaining `null` network/signer/checkpoint selected fields are open.

## Acceptance rules

The ledger becomes fully runnable only when:

- `profile_status` is `frozen`;
- every selected field is concrete;
- the fixture SHA-256 has been independently verified in CI;
- crypto sizes and vectors agree in both directions with the selected implementation;
- network codec and message-ID vectors pass exactly;
- the fingerprint has been recomputed from canonical data;
- no selected pin is derived only from peer behavior.

Any mismatch is an error. Multi-generation decoding, size-based crypto detection, automatic proof-version fallback, and persisted-state migration across fingerprints are forbidden unless a separately specified and tested migration tool performs an explicit offline conversion.
