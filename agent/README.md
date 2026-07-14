# Reference agent

**Status: design only, not yet implemented** (Months 3–5 milestone).

A reference off-chain service an anchor runs alongside its existing [Stellar
Anchor Platform](https://github.com/stellar/anchor-platform) deployment. It
extends the Anchor Platform's SEP-12 server with a hook that:

1. Reads the anchor's private KYC/cleared-user data store.
2. Generates the relevant Groth16 proof (via the circuits in `../circuits/`)
   for a given SEP-31 payment.
3. Submits the proof to `proof_verifier` and, where relevant, a new
   commitment to `attestation_registry`.
4. Encrypts the actual IVMS-101 payload to the counterpart anchor's public
   key and relays it, recording only a hash pointer via
   `travel_rule_envelope`.

## Why this isn't built yet

This depends on the circuits and contracts existing and having a real
testnet deployment first — building the agent before that would mean
building against an API surface that's still changing. See the repo-level
roadmap in `README.md` for sequencing.

## Intended integration point

The [Anchor Platform](https://github.com/stellar/anchor-platform) ships a
reference Kotlin SEP-12 server (`kotlin_reference_server` in that repo). The
plan is to extend that server with a proof-generation hook rather than
write a parallel SEP-12 implementation from scratch — anchors already
running the Anchor Platform should be able to adopt this as an add-on, not a
replacement. This choice is a direct response to the research finding that
Anchor Platform is explicitly scaffolding-only and expects exactly this kind
of extension.

## Open design questions

- Language: extend the existing Kotlin reference server directly, or run a
  separate sidecar service (any language) that the Kotlin server calls out
  to via a webhook? The sidecar approach decouples the proving logic from
  Anchor Platform's release cycle, at the cost of an extra network hop.
- Key management for the envelope encryption keys — where the anchor's
  decryption key is stored and rotated is a real operational security
  question, not just a code question.
