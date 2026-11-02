# Skip trivial Type-1 jobs (2026-11-02)

leanSpec [PR #748](https://github.com/leanEthereum/leanSpec/pull/748) (open)
and Zeam [#907](https://github.com/blockblaz/zeam/issues/907): a 1-validator
Type-1 costs a full STARK and cannot move a ⅔ quorum. The raw XMSS signature
is already on the attestation subnet.

Ethean now skips the prover when:

- there are no child proofs and at most one raw signature, or
- there are no raw signatures and fewer than two child proofs.

The signatures stay in the pool for the next round. Two raw votes, or a raw
vote plus a child, still prove.

The in-repo leanSpec fixture digest is unchanged.
