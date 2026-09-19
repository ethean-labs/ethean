# Charter

This directory defines why the Ethean Lean Consensus migration exists, which boundaries it must respect, and how completion is judged.

## Contents

- [PROJECT_CHARTER.md](./PROJECT_CHARTER.md) defines the mission, authority, governance, phases, and stop conditions.
- [SCOPE_AND_NON_GOALS.md](./SCOPE_AND_NON_GOALS.md) fixes the included product surface and rejects legacy or speculative expansion.
- [SUCCESS_CRITERIA.md](./SUCCESS_CRITERIA.md) defines measurable phase and program acceptance gates.

## Binding interpretation

The migration is a replacement program, not an incremental conversion of the Panro/Beacon implementation. “Reuse” means reusing a responsibility, lesson, or independently verified generic design in newly authored Ethean code. It never grants retain status to an old source file.

Any exact protocol value absent from an approved phase compatibility ledger is blocked in Phase 00. Implementers must not substitute a Beacon constant, a peer-client majority, a historical devnet value, or a convenient library default.
