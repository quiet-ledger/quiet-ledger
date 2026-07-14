# Threat model

**Status:** draft, month 1–2 deliverable. Expect this to change as circuits
and contracts are actually implemented — treat this as the starting
assumptions to build against and challenge, not a finished analysis.

## Assets being protected

1. **Raw PII** (names, addresses, identity documents) belonging to
   originators/beneficiaries of SEP-31 payments.
2. **The shape of an anchor's private cleared-user set** — even revealing
   *how many* users are in the set, or being able to test membership offline,
   is a smaller but real leak.
3. **Exact transaction amounts**, where only threshold-band information is
   supposed to be disclosed.

## Trust assumptions

- Each anchor is trusted to determine its own KYC/sanctions status for its
  users correctly — this protocol does not attempt to verify *that*
  determination, only to let an anchor disclose claims about it without
  over-sharing. A malicious or negligent anchor can still publish a false
  commitment; this is a policy/regulatory problem, not one this protocol's
  cryptography can solve alone.
- The off-chain proof-generation agent (`agent/`) runs in an environment the
  anchor controls and already has access to the raw KYC data it's proving
  claims about. It is not a multi-party computation between anchors.
- The travel-rule envelope's encryption keys are managed by each anchor;
  key compromise is out of scope for the protocol itself (standard key-
  management practices apply, tracked separately).

## Threats considered

| Threat | Mitigation |
|---|---|
| Public ledger leaks PII | Only proof outputs and hash pointers are ever written on-chain; the actual payload is encrypted and relayed off-chain directly between anchors. |
| Proof replay across unrelated transactions | Proofs bind to a specific transaction/escrow reference as a public input (exact binding scheme: open design issue). |
| Anchor publishes a stale/never-updated commitment to hide a revoked clearance | `attestation_registry` commitment refresh cadence is a protocol parameter (see RFC §5) — needs a bounded staleness window. |
| Counterpart anchor can't independently verify a claim | All verification happens via public on-chain contract calls (`proof_verifier`), not a private API call to the proving anchor. |
| Threshold circuit reveals amount via public-signal side channel | Public signals must only include the threshold comparison result, never a value correlated with the exact amount — tracked as a circuit-review checklist item before the external audit milestone. |

## Explicitly out of scope for v1

- Multi-party computation to distribute trust in the underlying KYC
  determination itself.
- Hiding *which* k-of-n institutional co-signers approved a threshold
  attestation (see `ARCHITECTURE.md`'s note on `threshold_attestation`).
- Formal verification of the Circom circuits (tracked as a future milestone,
  not this repo's initial scope).

## Before the external audit milestone (Months 6–8)

This document should be revisited and every "open design issue" reference
above should be resolved or explicitly deferred with reasoning, not left
dangling.
