# Scale flags exist; no large run on this host (2026-11-02)

Item 11 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

`scripts/local-devnet.ps1` already takes `-Nodes`, `-ValidatorsPerNode`,
`-Subnets` (`ATTESTATION_COMMITTEE_COUNT`), and `-Aggregators`. Details:
[scale-multi-aggregator-2026-09-30.md](./scale-multi-aggregator-2026-09-30.md).

This host did not start a larger layout (CPU heat). Multi-host / leanstart
still need Docker and a public image. Do not invent A2/A3.
