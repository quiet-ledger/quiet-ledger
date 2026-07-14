# Architecture

## Overview

```
                    ┌─────────────────────┐
                    │   Anchor A (sender)  │
                    │  private KYC store   │
                    └──────────┬───────────┘
                               │ generates proofs from private data
                               ▼
                    ┌─────────────────────┐
                    │  agent/  (off-chain) │  extends Anchor Platform's SEP-12 server
                    └──────────┬───────────┘
                               │ submits proof + commitment
                               ▼
        ┌──────────────────────────────────────────────┐
        │                Soroban contracts               │
        │  attestation_registry   proof_verifier          │
        │  (per-anchor commitments) (wraps zksoroban's    │
        │                            verify_proof)        │
        │            travel_rule_envelope                 │
        │       (hashed pointer to encrypted PII)          │
        └──────────────────────┬───────────────────────┘
                               │ envelope pointer + encrypted relay
                               ▼
                    ┌─────────────────────┐
                    │  Anchor B (receiver) │
                    │  agent/ decrypts via │
                    │  its own private key │
                    └─────────────────────┘
```

## Components

### `circuits/`

Three Circom circuits, each proving a specific disclosure claim without
revealing the underlying data:

- **`merkle_membership`** — proves a private leaf (e.g. a hashed user
  identifier) is included in a Merkle tree whose root is public (e.g. an
  anchor's "cleared" set), without revealing the leaf, the path, or the rest
  of the tree.
- **`range_disclosure`** — proves a private value (e.g. transaction amount)
  falls below (or above) a public threshold, without revealing the value
  itself. Implemented as a bit-decomposition range check — the simplest
  correct pattern for this and the only circuit in this repo that is fully
  implemented today (see `circuits/range_disclosure/README.md`).
- **`threshold_attestation`** — a k-of-n joint sign-off from institutional
  co-signers. On reflection during scaffolding, this does **not** need to be
  a ZK circuit at all in its base form: verifying that k-of-n signatures
  over an attestation are valid is a standard multisig check, which Soroban
  can do natively via its auth framework. A circuit is only needed for the
  strictly harder variant of hiding *which* k-of-n signed — that variant is
  out of scope until an issue specifically asks for it. See
  `circuits/threshold_attestation/README.md`.

### `contracts/`

- **`attestation_registry`** — stores each anchor's current commitment roots
  (e.g. the Merkle root of its cleared-user set) and rotation history.
- **`proof_verifier`** — verifies proofs against the registry's commitments.
  Wraps the existing generic Groth16 verifier from
  [`zksoroban`](https://github.com/iamkingvalor/zksoroban)'s
  `contracts/verifier` rather than reimplementing Groth16-on-Soroban
  verification from scratch.
- **`travel_rule_envelope`** — stores only a hash pointer to the actual
  encrypted travel-rule payload (originator/beneficiary IVMS-101 data), which
  is exchanged directly between anchors off-chain, never on the public
  ledger.

### `sdk/`

TypeScript and Rust clients for anchors to generate proofs from their private
KYC store and submit them to the contracts above.

### `agent/`

A reference off-chain service extending the [Stellar Anchor
Platform](https://github.com/stellar/anchor-platform)'s SEP-12 server with a
proof-generation hook, so an anchor can adopt this without replacing its
existing Anchor Platform deployment.

### `e2e/`

Idempotent `stellar-cli` scripts that deploy to testnet and exercise the full
flow (registry commitment → proof submission → verification → envelope
exchange), following the same pattern used in this account's other Soroban
projects: shared `lib.sh` helpers, numbered scripts, state assertions after
every step.

## Trust assumptions

See `docs/THREAT_MODEL.md` for the full analysis. Summary: this protocol
assumes the off-chain proof-generation agent is run by a party that already
holds the private KYC data (the anchor itself) — it does not attempt to
distribute trust in *generating* the underlying KYC determination, only in
*disclosing* claims about it without over-sharing.
