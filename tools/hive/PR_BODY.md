## Summary

Register **Ethean** as a Lean Consensus client in the Hive lean simulator.

- Add `clients/ethean/` (wrapper of `ghcr.io/ethean-labs/ethean:devnet5`,
  source-build `Dockerfile.git`, Ream-shaped entrypoint + validators).
- Matrix: `ethean=devnet4,devnet5` in `lean-devnets.txt`.
- Client lists: `devnet4.yaml` / `devnet5.yaml`.
- Runtime asset prep: treat `ethean` like `ream` (dual-key registry + multiaddr
  bootnodes, not ENR).

Source drop-in lives in the Ethean repo:
https://github.com/ethean-labs/ethean/tree/master/docker/hive/upstream-clients-ethean

Apply helper (idempotent):
`ethean/tools/hive/apply-ethean-client.sh /path/to/hive`

## Test plan

- [ ] `ghcr.io/ethean-labs/ethean:devnet5` pulls successfully
- [ ] `./hive --sim lean --client-file simulators/lean/clients/devnet5.yaml --client ethean --docker.output`
- [ ] Health `GET /lean/v0/health` returns `healthy` + `version`
- [ ] Smoke against one peer client (e.g. ream) on the same lean sim profile
