# RFC v0.1: Quiet Ledger travel-rule disclosure protocol

**Status:** draft. Not yet submitted as a formal Stellar SEP proposal — that
is a later roadmap milestone (Months 9–10), pending pilot feedback.

## 1. Motivation

SEP-31 requires the sending anchor to transmit originator/beneficiary
information to the receiving anchor for compliance purposes. Today this
either happens out-of-band with no standard format, or not at all. This RFC
defines a protocol for anchors to exchange *proofs about* required disclosure
claims — sanctions clearance, threshold-based reporting obligations, joint
institutional sign-off — without requiring the counterpart to trust a single
party's say-so, and without broadcasting PII on a public ledger.

## 2. Terminology

- **Anchor** — a SEP-31 sending or receiving anchor, as defined in the
  Stellar SEP process.
- **Commitment** — a Merkle root or similar cryptographic commitment an
  anchor publishes to `attestation_registry`, representing a private set
  (e.g. "users I've cleared KYC for") without revealing its members.
- **Envelope** — the `travel_rule_envelope` record: a hash pointer to an
  encrypted off-chain payload containing the actual IVMS-101-shaped
  travel-rule data.

## 3. Protocol flow

1. Anchor A maintains a private "cleared users" set and periodically
   publishes its Merkle root to `attestation_registry`.
2. When Anchor A processes a SEP-31 payment for user U, its `agent/` service
   generates a `merkle_membership` proof that U is in the cleared set, and (if
   the amount crosses the reporting threshold) a `range_disclosure` proof
   about the amount band.
3. Anchor A submits these proofs to `proof_verifier`, which checks them
   against Anchor A's published commitment.
4. Anchor A encrypts the actual IVMS-101 payload to Anchor B's public key and
   sends it via the relay; only a hash pointer goes into
   `travel_rule_envelope`.
5. Anchor B independently verifies the on-chain proof outputs match what it
   expects before decrypting and accepting the payload.

## 4. Data model (IVMS-101 mapping)

The off-chain envelope payload maps to a subset of
[IVMS-101](https://intervasp.org/) originator/beneficiary fields. Exact field
mapping is tracked as an open design issue — draft the mapping table here as
that work lands.

## 5. Open questions

- Threshold configuration: does the protocol hardcode the FATF USD/EUR 1,000
  threshold, or make it a per-anchor configurable parameter enforced by
  `range_disclosure`? Current lean: configurable, since thresholds vary by
  jurisdiction.
- Key rotation for the envelope encryption keys.
- Whether `attestation_registry` commitments should have a mandatory
  refresh/expiry window to bound how stale a "cleared" claim can be.

## 6. Non-goals

This RFC does not attempt to replace an anchor's underlying KYC/AML
determination process — it only standardizes how a *result* of that process
is disclosed to a counterpart anchor.
