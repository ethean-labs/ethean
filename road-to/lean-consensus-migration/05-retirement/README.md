# Retirement and Repository Cleanup

This directory defines the final removal gate for the Lean Consensus migration. Retirement is complete only when replaced implementation, stale planning material, obsolete names, and empty legacy directories are removed from the active repository surface.

## Documents

- [DELETION_REGISTER.md](DELETION_REGISTER.md) inventories current source and documentation classes, their replacements, prerequisites, and verification searches.
- [REQUIRED_DIRECTORY_POLICY.md](REQUIRED_DIRECTORY_POLICY.md) defines which top-level directories must remain or be recreated and how they are documented.
- [LEGACY_NAME_ALLOWLIST.md](LEGACY_NAME_ALLOWLIST.md) defines the only contexts in which historical names may remain.

## Retirement sequence

1. Land the target workspace skeleton and crate READMEs.
2. Move behavior behind the target crate APIs without compatibility dependencies back into the old root package.
3. Prove feature, test, and operational parity for each deletion-register row.
4. Delete replaced source files and manifests in the same phase that activates their replacement.
5. Reclassify or delete old roadmap and development-status documents.
6. Run repository-wide legacy-name and obsolete-path scans.
7. Remove every empty obsolete directory.
8. Recreate required top-level directories that became empty and add an English `README.md`.
9. Validate workspace metadata, formatting, linting, tests, documentation links, source-size policy, and packaging contents.

## Definition of done

- The root is a virtual workspace and has no historical package target.
- Active code exists only in `crates/` and `bin/ethean/`, apart from integration tests, examples, benches, and tooling in their required top-level directories.
- No obsolete source directory or empty compatibility directory remains.
- Every retained historical document is clearly classified as history and is not linked as current guidance.
- Active code, configuration, APIs, logs, metrics, tests, examples, and current documentation contain no legacy names.
- Required top-level directories exist and include English READMEs.
- All deletion-register verification commands return the expected empty result.

Deletion is not optional cleanup. Keeping unused modules “for reference,” retaining empty folder shells, or leaving aliases that preserve obsolete naming fails this phase.
