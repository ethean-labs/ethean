# Legacy Name Allowlist

## Purpose

Ethean Lean Consensus Client retires **Panro**, legacy **Beam** branding, **Beacon Chain**, **Eth2**, and pre-Lean **BLS/WOTS** naming from every active surface. Historical names may appear only in explicitly allowlisted baseline and retirement documentation.

This policy complements [DELETION_REGISTER.md](./DELETION_REGISTER.md) and [01-baseline/CURRENT_STATE_AUDIT.md](../01-baseline/CURRENT_STATE_AUDIT.md).

---

## Allowed contexts (exhaustive)

Legacy names (`panro`, `Panro`, `beam`, `Beam`, `beam-chain`, `Beacon`, `beacon`, `Eth2`, `eth2`, `BLS`, `bls`, `WOTS`, `wots`, and common compounds like `BeaconBlock`, `panro-labs`) may appear **only** in:

| Allowlisted path | Permitted use |
| --- | --- |
| `road-to/lean-consensus-migration/01-baseline/**` | Describing the audited old repository state |
| `road-to/lean-consensus-migration/05-retirement/**` | Deletion inventory, verification commands, allowlist itself |
| Quoted historical filenames inside retirement docs | e.g. `` `road-to/panro-roadmap-vision.md` `` as a deletion target |
| Quoted code/path literals in baseline or retirement docs | Showing what to search for or delete |
| Git commit messages and git history | Not edited retroactively |

No other tracked path may contain these names in prose, identifiers, configuration, or user-facing strings.

---

## Forbidden contexts (must be clean)

The following surfaces must contain **zero** legacy-name hits (case-insensitive, with compound patterns below):

| Surface | Examples of violations |
| --- | --- |
| Active Rust source | `PanroClient`, `mod bls`, `BeaconBlock`, crate `panro` |
| `Cargo.toml` / manifests | `name = "panro"`, BLS dependency comments tied to product |
| CLI flags, env vars, defaults | `--network panro`, `BEAM_CHAIN` |
| API routes, types, OpenAPI | `/eth/v1/beacon`, `BeaconStateResponse` |
| Log messages and tracing targets | `panro::network`, `Beacon block imported` |
| Metrics and labels | `panro_head_slot`, label `chain=eth2` |
| Tests, fixtures, examples | assert messages, example titles referencing Panro |
| Current documentation | `docs/**` except when quoting deletion targets in migration summaries that point at retirement docs |
| Deployment and observability configs | service names, dashboard titles |
| Root `README.md` | product must be **Ethean Lean Consensus Client** |

Historical **quotes** inside allowlisted docs must be clearly marked as historical inventory, not current product naming.

---

## Search gates

Run from repository root. **Any hit outside allowlisted paths fails R5** unless the hit is a quoted literal documenting a deletion target inside an allowlisted file (manual review).

### Gate 1 — Product and chain identity

```powershell
rg -n -i '\bpanro\b|panro-labs|beam[ -]?chain|\bbeam\b|\bbeacon\b|\beth2\b' `
  --glob '!road-to/lean-consensus-migration/01-baseline/**' `
  --glob '!road-to/lean-consensus-migration/05-retirement/**' `
  --glob '!*.lock' `
  .
```

Expected: **no matches**.

### Gate 2 — Cryptography legacy (BLS/WOTS)

```powershell
rg -n -i '\bbls(t|12|trs)?\b|\bwots\b|blst|blstrs|bls12_381' `
  --glob '!road-to/lean-consensus-migration/01-baseline/**' `
  --glob '!road-to/lean-consensus-migration/05-retirement/**' `
  crates bin tests examples benches docs README.md Cargo.toml
```

Expected: **no matches** on active surfaces. Migration protocol docs discussing “do not use BLS” belong in `02-protocol/` without embedding legacy crate names unless quoting baseline evidence — prefer “legacy Beacon cryptography” wording there.

### Gate 3 — Obsolete source paths

```powershell
rg -n 'src/(api|bench|bin|config|consensus|crypto|integration|network|optimization|storage|types|utils)|PanroClient' `
  --glob '!road-to/lean-consensus-migration/01-baseline/**' `
  --glob '!road-to/lean-consensus-migration/05-retirement/**' `
  crates bin tests examples benches docs README.md Cargo.toml
```

Expected: **no matches**.

### Gate 4 — Examples and binaries

```powershell
rg -n -i 'panro|beam|beacon|eth2|bls|wots|src::' examples bin
```

Expected: **no matches**.

### Gate 5 — Road-to outside migration (R5 target)

```powershell
rg -n -i 'panro|beam[ -]?chain|beacon|eth2' road-to `
  --glob '!lean-consensus-migration/**'
```

Expected: **no matches** after R5 deletes or archives listed legacy planning files.

### Gate 6 — Package name in workspace

```powershell
rg -n 'name\s*=\s*"panro"|default-run\s*=\s*"panro"' Cargo.toml bin/*/Cargo.toml crates/*/Cargo.toml
```

Expected: **no matches** after R1.

---

## CI integration (recommended)

Add a repository script (future `tools/legacy-name-gate.ps1` or Rust xtask) that:

1. runs Gates 1–6;
2. prints unified diff of hits with file paths;
3. exits non-zero on any non-allowlisted hit;
4. runs in pre-merge CI alongside `cargo fmt`, `clippy`, and tests.

Until the script exists, R5 manual review uses the commands above.

---

## Handling false positives

| Hit type | Action |
| --- | --- |
| Substring in unrelated word | Rare; rewrite context or use word boundaries in rg |
| Third-party dependency name in `Cargo.lock` | Lockfile excluded from Gate 1; dependency must still be removed from manifest in R2 |
| Quoted deletion path in `02-protocol/` | Rephrase to avoid legacy identifiers unless essential; prefer linking to this allowlist |
| Username/email in old git history | Ignored — do not rewrite history |

---

## Definition of clean

R5 **legacy-name clean** means:

- Gates 1, 3, 4, 5, and 6 return empty;
- Gate 2 returns empty on active code/docs manifests;
- Every remaining historical mention lives under `01-baseline/` or `05-retirement/` and reads as inventory, not product identity;
- [DELETION_REGISTER.md](./DELETION_REGISTER.md) final verification passes.

Product-facing name everywhere else: **Ethean Lean Consensus Client** (crate/binary: `ethean`, workspace crates per [TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md)).
