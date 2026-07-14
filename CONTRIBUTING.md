# Contributing

Contributions should preserve the core guarantees of this repository:

- a proof must never leak more than the specific claim it's designed to disclose
- the on-chain verifier, the circuits, and the SDKs must stay byte-for-byte interoperable
- nothing that could plausibly be raw PII gets written on-chain, even hashed, without a documented justification in `docs/THREAT_MODEL.md`

## Known toolchain issue: `cargo test` currently fails to compile

As of this writing, `cargo test` (and `cargo clippy --all-targets`) fails
during dependency compilation with an unrelated trait-bound error inside
`soroban-env-host`'s `testutils` feature (`ChaCha20Rng` vs.
`ed25519_dalek::rand_core::CryptoRng`) — a version-skew bug between
`ed25519-dalek` 3.0.0 and `rand_chacha`'s pinned `rand_core`, upstream in
`soroban-env-host 25.0.1`, not in this repo's code. It reproduces identically
on [zksoroban's own `contracts/verifier`](https://github.com/iamkingvalor/zksoroban)
when tested fresh (without its committed lockfile), confirming it isn't
something introduced here. Plain `cargo build` and `cargo clippy` (without
`--all-targets`) are unaffected and pass clean — that's how this repo's
contract logic was verified pending an upstream fix. Tracked as an open
issue; check whether a newer `soroban-sdk`/`soroban-env-host` patch release
has resolved it before spending time re-diagnosing.

## Development expectations

Before submitting changes:

1. Run contract tests: `cargo test` (or `make check` for fmt + clippy + test)
   — see the toolchain note above if this fails to compile in your
   environment.
2. If you touch a circuit, rebuild its artifacts per that circuit's `README.md` and confirm the on-chain verifier's expected public-signal count still matches.
3. Run the SDK tests: `sdk/ts` (`npm test`) and `sdk/rust` (`cargo test`).
4. Run the e2e suite against testnet where feasible: `cd e2e && ./run_all.sh`.
5. If you change proof structure, public-signal ordering, or the travel-rule envelope schema, update `docs/RFC.md` and `ARCHITECTURE.md` in the same PR.

## Scope notes

- This is pre-alpha. Nothing here should be treated as audited or
  production-ready until the external audit milestone lands.
- The threshold-attestation piece is deliberately scoped as an on-chain
  k-of-n signature check first (see `circuits/threshold_attestation/README.md`)
  — don't add ZK-hiding of *which* signers co-signed unless an issue
  specifically asks for it; it's a materially harder, separate problem.

## Pull request guidance

Good changes: circuit correctness fixes, verifier/registry contract
improvements, SDK interoperability work, test coverage, documentation fixes
tied to actual behavior, e2e script improvements.

Changes that need extra care and should start as a discussion issue first:
changing public-signal encoding, changing which data is committed on-chain
versus kept off-chain, changing the travel-rule envelope's data model, or
anything that touches the threat model's trust assumptions.

## Picking up an issue

Every open task uses the "Wave task" issue template with explicit acceptance
criteria and a difficulty rating (`easy` / `medium` / `hard`). Issues labeled
`good first issue` are scoped to be self-contained — comment on the issue
before starting so work doesn't collide.
