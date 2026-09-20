<div align="center">
  <img width="220" alt="Ethean" src="https://github.com/user-attachments/assets/dfd84be4-8acf-441d-bada-8329b99c8c27" />
</div>

# Ethean

**Research & engineering for the next era of Ethereum consensus**

---

### Who We Are

Ethean is a research and engineering collective rooted in blockchain protocol work.

Our people have spent years designing, implementing, and operating core pieces of multiple chains — consensus, peer-to-peer networking, cryptography, client architecture, and production distributed systems. That background is not a slogan; it is the filter we bring to hard protocol decisions.

We are not a new team chasing a trend. We are engineers and researchers who have already walked protocol work from early design through real deployments, and who now apply that experience to a focused, ambitious problem.

---

### Our Focus

Our primary effort is **Lean Consensus** — the post-quantum redesign of Ethereum’s consensus layer (also discussed as Beam / leanEthereum).

We are building a modular, high-performance **consensus client in Rust** that tracks the evolving Lean research tracks and pq-devnet lessons. The work sits under the broader Ethereum research umbrella: contribute usable client diversity, not another frozen Beacon clone.

Lean Consensus is a clean-slate chance to rethink the pieces that matter most for the next decade:

- **Post-quantum signatures** — hash-based schemes (leanSig / XMSS-style), not long-term BLS dependence
- **Aggregation at scale** — leanMultisig and later recursive / zkVM-backed aggregates as validator counts grow
- **Faster finality** — seconds-scale finality (3SF today; PQ heartbeat / Goldfish-style direction ahead)
- **Tighter networking** — ~4s slots, QUIC, Gossipsub evolution, and the P2P load of a much larger validator set
- **Validator economics** — roadmap pressure toward lower stake floors (e.g. toward 1 ETH), which changes client and network design

We treat this as one of the most important protocol efforts in the Ethereum ecosystem over the coming years — and we intend to meet it with rigorous engineering, not demos.

---

### What Guides Us

- **Long-term thinking** — Correctness, auditability, and sustainability over short-term milestones.
- **Protocol craftsmanship** — Small, careful decisions compound into systems that survive contact with reality.
- **Client diversity** — A healthy Ethereum needs multiple independent, high-quality implementations. We aim to be one of them.
- **Interop over imitation** — We watch peer Lean clients and leanSpec / leanroadmap pins for protocol alignment; we keep our own architecture and house style.
- **Open collaboration** — We work in the open and welcome engagement from Ethereum research and client communities.
- **Continuity of expertise** — The same people who have delivered complex blockchain systems before are applying that knowledge to Lean Consensus.

---

### Our Path

We have already invested substantial time across multiple blockchain protocols. That history gives a practical sense of constraints that only appear after years of development and operation — not just in whitepapers.

With Ethean we channel that experience into a deliberate client: lean in scope (consensus, not execution), modular in layout, and aligned with the post-quantum direction of Ethereum research — from local finality and pq-devnet readiness toward a client that can stand beside its peers on the mesh.

This is not a side project or a prototype thrown together by a new team. It is the next chapter of sustained protocol work by engineers who have lived the full lifecycle of blockchain systems.

---

### Looking Ahead

Ethereum’s consensus layer is entering a new phase. Lean Consensus offers a rare chance to redesign critical components with the benefit of operational lessons already paid for.

Ethean exists to help make that transition succeed — through careful engineering, thorough research, and a commitment to the long-term health of the Ethereum protocol.

We are here for the long run.

---

**Ethean**  
*Continuing protocol development under the Ethereum umbrella.*

---

##  Support
- **Documentation**: [docs](./docs/)
- **Docs index**: [docs/README.md](./docs/README.md)
- **Folder READMEs and local conventions**: [docs/folder-readmes-and-local-conventions.md](./docs/folder-readmes-and-local-conventions.md)
- **Lean Consensus migration plans**: [road-to/lean-consensus-migration/README.md](./road-to/lean-consensus-migration/README.md) (active planning library; see also [road-to/README.md](./road-to/README.md))
- **Source tree**: [src/README.md](./src/README.md)
- **Lean Consensus tracks**: [leanroadmap.org research tracks](https://leanroadmap.org/#research-tracks)
- **Lean Consensus R&D (full site)**: [leanroadmap.org](https://leanroadmap.org/)
- **How we capture that locally**: [docs/leanroadmap-local-notes.md](./docs/leanroadmap-local-notes.md)
- **Source file size (300 lines)**: [docs/source-file-size-limit.md](./docs/source-file-size-limit.md)
- **Peer Lean clients (reference)**: [docs/peer-reference-clients.md](./docs/peer-reference-clients.md)
- **How Ream / ethlambda / Zeam run pq-devnets**: [docs/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](./docs/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
- **Peer fixed genesis vs Ethean solo restart**: [docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](./docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md)
- **Dual mode (persist + ephemeral)**: [docs/dual-mode-persist-and-ephemeral-2026-09-20.md](./docs/dual-mode-persist-and-ephemeral-2026-09-20.md)
- **Full State SSZ encode/decode**: [docs/state-ssz-encode-decode-complete-2026-09-20.md](./docs/state-ssz-encode-decode-complete-2026-09-20.md)
- **Seven-client source research**: [docs/lean-peer-client-research-library-2026-09-19.md](./docs/lean-peer-client-research-library-2026-09-19.md)
- **Language (English only)**: [docs/english.md](./docs/english.md)
- **Commits (per file, English)**: [docs/commit-after-each-file.md](./docs/commit-after-each-file.md)
- **GitHub Issues**: [Report bugs](https://github.com/Pamenarti/Ethean/issues)
- **Email**: support@Ethean.io

---

<div align="center">

**Built with ❤️ by the Ethean Team**

[Website](https://Ethean.io) • [GitHub](https://github.com/Pamenarti/Ethean)

</div>

---

**Note**: This is a development version. For production use, please wait for the stable release and conduct thorough testing in your environment.
