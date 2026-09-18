# Required Directory Policy

## Purpose

Retirement removes obsolete implementation trees, but the repository must keep a predictable top-level layout for code, tests, examples, benchmarks, documentation, planning, deployment notes, and observability contracts. A required directory that loses its last substantive file is **recreated in the same change** with an English `README.md`. Required directories are never absent or empty.

Non-required obsolete directories are deleted completely; see [DELETION_REGISTER.md](./DELETION_REGISTER.md).

---

## Required top-level directories

| Directory | When required | Must contain | Prohibited contents |
| --- | --- | --- | --- |
| `crates/` | Always after R1 | One folder per library crate, workspace `README.md` | Executable composition, monolithic `src/` mirrors, unrelated tools |
| `bin/` | Always after R1 | `ethean/` executable package, `README.md` | Reusable consensus logic, duplicate library modules, retired binary names |
| `tests/` | Always after R1 | Cross-crate and E2E tests, `README.md` | Production modules, private copies of crate internals |
| `examples/` | Always | Compiling public-API examples, `README.md` | Legacy `src::` examples, disabled reference code |
| `benches/` | When benchmarks exist | Harnesses and/or pointer to crate-local benches, `README.md` | Unverified performance claims, production runtime |
| `docs/` | Always | Current user/operator/contributor docs, `README.md` index | Unclassified weekly reports presented as current guidance |
| `road-to/` | Always during migration | Migration plan, classified history, `README.md` | Active source, generated artifacts, scratch notes (`notlar.txt`) |
| `deploy/` | When deployment artifacts or runbooks exist | Environment templates, compose/k8s manifests **or** operator runbooks, `README.md` | Secrets, `.env`, production keys, unreviewed scripts |
| `.github/` | When CI/automation exists | Workflows, `README.md` describing ownership and security boundaries | Empty placeholder without workflows |

### Observability (as needed)

Observability is not always a top-level directory. It is required in one of these forms:

| Location | When | Must contain |
| --- | --- | --- |
| `deploy/observability/` | Recommended when shipping dashboards, alert rules, or scrape configs | Prometheus/Grafana (or equivalent) assets + `README.md` |
| `crates/metrics/` | Always in target workspace | Metric emission implementation + crate `README.md` |
| `road-to/lean-consensus-migration/06-observability/` | During migration planning | [METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md), observability gates |

When `deploy/observability/` is created or touched, it needs an English `README.md` explaining scrape targets, label policy ([METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md)), and what must not be shipped (raw peer IDs, keys, roots as labels).

---

## Required package directories (post-R1)

```text
crates/primitives/
crates/profile/
crates/types/
crates/crypto/
crates/transition/
crates/fork-choice/
crates/storage/
crates/network-wire/
crates/network/
crates/sync/
crates/validator/
crates/node/
crates/rpc/
crates/metrics/
bin/ethean/
```

Each package root includes:

- `Cargo.toml`;
- English `README.md` (responsibility, public entry points, dependencies, I/O ownership, exclusions, architecture link);
- `src/` with `lib.rs` (library) or `main.rs` (executable).

Optional `tests/`, `benches/`, `examples/` subdirs are created only with real files and local README when the folder holds more than a single trivial file.

---

## README contract

Every directory that holds code, tests, examples, benchmarks, documentation, planning, deployment, observability assets, or tools must have an English `README.md`.

Organizational READMEs state:

1. purpose and retention policy;
2. naming conventions;
3. index of important children;
4. what must not live there.

READMEs must not describe unimplemented behavior as current.

---

## Preserve, recreate, or remove

| Situation | Action |
| --- | --- |
| Directory still required and current | **Preserve**; update README when responsibility shifts |
| Required directory lost its last file during migration | **Recreate** immediately with README in the same commit |
| Directory listed in deletion register and fully migrated | **Remove**; verify with register search commands |
| Directory exists only to preserve old links | **Remove**; fix links to target paths |
| Temptation to use `.gitkeep` | **Forbidden** — README is the meaningful tracked file |

### Old `src/` rule

The historical top-level `src/` is **not required** after workspace migration. It must be deleted when [DELETION_REGISTER.md](./DELETION_REGISTER.md) rows close. Source lives in package-local `src/` directories only. Do not recreate `src/types`, `src/consensus`, etc., under another compatibility root.

---

## Naming and language

- New directory and file names: lowercase kebab-case except ecosystem standards (`README.md`, `Cargo.toml`, Rust module files).
- All tracked content: English ([project english-only rule](../../../.cursor/rules/09-english-only.mdc)).
- Directory names must not contain retired product names (`panro`, legacy `beam` branding). See [LEGACY_NAME_ALLOWLIST.md](./LEGACY_NAME_ALLOWLIST.md).

---

## Verification commands

Run during R5 and on every retirement batch:

```powershell
# 1. Required top-level dirs present
$required = @('crates','bin','tests','examples','docs','road-to')
foreach ($d in $required) {
  if (-not (Test-Path $d)) { throw "Missing required directory: $d" }
  if (-not (Test-Path "$d/README.md")) { throw "Missing README: $d/README.md" }
}

# 2. deploy/ and deploy/observability/ when used
if (Test-Path deploy) {
  if (-not (Test-Path deploy/README.md)) { throw "deploy/ requires README.md" }
}
if (Test-Path deploy/observability) {
  if (-not (Test-Path deploy/observability/README.md)) {
    throw "deploy/observability/ requires README.md"
  }
}

# 3. No empty directories
$empty = Get-ChildItem -Directory -Recurse |
  Where-Object { -not (Get-ChildItem $_.FullName -Force) }
if ($empty) {
  $empty.FullName
  throw "Empty directories are prohibited."
}

# 4. Obsolete src/ gone
if (Test-Path src) { throw "Obsolete src/ must be deleted" }

# 5. Package README coverage (after workspace exists)
Get-ChildItem crates,bin -Directory | ForEach-Object {
  if (-not (Test-Path (Join-Path $_.FullName 'README.md'))) {
    throw "Missing package README: $($_.FullName)"
  }
}
```

Also validate relative Markdown links and packaged files so planning notes, local research paths, and generated artifacts are not shipped inside Rust crate packages.

---

## Relationship to deletion register

1. Delete obsolete paths per [DELETION_REGISTER.md](./DELETION_REGISTER.md).
2. If deletion removes the last file from a **required** directory, recreate the directory with a README explaining scheduled work.
3. If deletion removes a **non-required** directory, ensure no empty parent remains (collapse empty shells).
4. Run legacy-name scans from [LEGACY_NAME_ALLOWLIST.md](./LEGACY_NAME_ALLOWLIST.md) after structural changes.
