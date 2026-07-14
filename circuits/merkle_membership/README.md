# Merkle Membership Circuit

**Status: design only, no `circuit.circom` yet.** Deliberately so: a
half-working circuit with a placeholder hash is easy to mistake for real
crypto later. This README is the full spec an open issue (see the repo's
seeded issues) should implement against, once a reviewed ZK-friendly hash is
wired in (see below).

## What it proves

That a private `leaf` (e.g. a hash of a cleared user's identifier) is
included in a Merkle tree whose `root` is public, given a private
authentication path, without revealing the leaf, the path, or any other
member of the tree.

Inputs:

- private: `leaf`, `pathElements[depth]`, `pathIndices[depth]`
- public: `root`

Output:

- public: `isMember` (or the circuit can be structured to constrain
  `computedRoot === root` directly rather than emit a boolean — open
  design question, see below)

## Why this isn't implemented yet

A real Merkle-inclusion circuit needs a ZK-friendly hash function inside the
circuit (Poseidon is the standard choice — same family already used by this
account's `poseidon_preimage` reference circuit). Implementing Poseidon
correctly and securely from scratch, uninspected, would be worse than not
having a circuit at all. The honest path is: either vendor a reviewed
Poseidon implementation (e.g. from circomlib) as a dependency, or port the
parameters already documented in `zksoroban`'s
[`docs/poseidon-parameters.md`](https://github.com/iamkingvalor/zksoroban/blob/main/docs/poseidon-parameters.md).
Both are tracked as the same open issue — do one, not a fresh untested hash
construction.

## Open design questions

- Fixed tree depth (e.g. 20, supporting ~1M cleared users) vs. variable
  depth — fixed is simpler and is the default assumption until an issue
  says otherwise.
- Whether the circuit should output a boolean `isMember` signal or directly
  constrain the recomputed root to equal the public `root` (the latter is
  tighter — a malformed proof simply fails to satisfy constraints rather
  than producing a false `isMember=0` that still verifies).

## Rebuilding (once implemented)

Same flow as `circuits/range_disclosure`: `circom circuit.circom --r1cs
--wasm --sym -o build`.
