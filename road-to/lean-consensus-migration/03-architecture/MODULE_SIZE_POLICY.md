# Module Size Policy

## Hard limit

Every hand-written source file is limited to 300 physical lines. This includes Rust, build scripts, shell scripts, configuration generators, test helpers, examples, and other authored code. Blank lines, comments, inline unit tests, and conditional-compilation sections count. Generated files, lockfiles, vendored sources, and machine-produced fixtures are exempt only when clearly marked and never manually edited.

The limit applies immediately when a file is created or edited. A pre-existing oversized file must be split in the same change that touches it.

## Required structure

Each module has one responsibility that can be named without “and.” `mod.rs` and `lib.rs` files declare modules, expose a deliberate public surface, and contain only small glue code. They are not storage locations for unclassified implementation.

Preferred decomposition:

```text
src/
  import/
    mod.rs
    validate.rs
    execute.rs
    persist.rs
    receipt.rs
```

Files are named after responsibilities such as `validate.rs`, `codec.rs`, `recovery.rs`, or `peer_score.rs`. Numeric suffixes, `misc.rs`, `helpers.rs`, `common.rs`, and `utils.rs` are prohibited unless the name describes a genuinely stable domain concept.

## Split triggers

A split is required before 300 lines and should normally begin around 240 lines when:

- a second independent responsibility appears;
- production code and extensive tests compete for space;
- multiple protocol versions require separate implementations;
- platform or backend conditionals obscure the main path;
- a module has distinct parsing, validation, execution, and persistence stages; or
- reviewers must scroll across unrelated types to understand a change.

The 300-line ceiling is not a target. Small cohesive files are preferred.

## Tests

Short unit tests may remain beside implementation. Larger suites move to a sibling `tests.rs` or a `tests/` module with files named by behavior. Integration tests live in the owning package's `tests/` directory. Test files obey the same 300-line limit and dependency rules as production files.

Moving tests must not expose private implementation solely for test access. Prefer testing public behavior, a narrow `pub(crate)` seam, or a test-only constructor in the owning module.

## Public API after a split

Splitting a file must preserve intentional caller-facing paths:

1. Add named child modules.
2. Keep implementation modules private unless direct use is part of the contract.
3. Re-export public types from the package or feature facade.
4. Remove accidental exports rather than duplicating them.
5. Update rustdoc links and tests to use supported paths.

Circular module imports are treated as a design defect. Shared value types move to the nearest lower-level domain module; orchestration remains in the parent.

## Directory documentation

Folder `README.md` files are optional. Do not create one for every new
directory. When a folder README exists, it may state:

- the directory's responsibility;
- its permitted dependencies or parent package;
- the kinds of files that belong there; and
- the kinds of logic that must live elsewhere.

A README is documentation and is not counted as source code, but it must remain concise enough to stay useful.

## Enforcement

CI enumerates tracked hand-written source files and counts physical lines. It fails with the path, observed count, and 300-line limit. The check must:

- use a reviewed extension allowlist;
- exclude only explicit generated/vendor/lockfile paths;
- inspect examples, benches, tests, and build scripts;
- run on all changed files and periodically on the complete repository; and
- reject newly added exemption patterns unless architecture owners approve them.

The migration acceptance check also reports the largest files so modules approaching the limit are visible before they block work.

## Review checklist

- Does each file have one clear responsibility?
- Is every touched hand-written source at or below 300 lines?
- Did a split preserve the supported public API?
- Are imports directed rather than circular?
- Did tests move with the behavior they verify?
- Does every new directory have an English README?
- Were generated or vendor exemptions justified by path and provenance?

Waivers are not permitted. If an implementation cannot fit, the architecture must expose the missing responsibility and split it into a named module.
