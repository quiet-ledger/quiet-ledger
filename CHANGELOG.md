# Changelog

All notable changes to this project are documented here. Format loosely
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Initial repository scaffold: circuit, contract, SDK, agent, and e2e
  directory structure.
- Protocol RFC draft (`docs/RFC.md`) and threat model draft
  (`docs/THREAT_MODEL.md`).
- `range_disclosure` circuit: working bit-decomposition range-check circuit.
- `merkle_membership` circuit design doc (constraint logic tracked as an
  open issue — see Roadmap in `README.md`).
- `threshold_attestation` contract: k-of-n institutional multisig check,
  deployed and initialized on testnet.
- All four contracts deployed and verified on Stellar Testnet with a
  passing `e2e/` suite.
- TypeScript SDK (`sdk/ts`): real implementation against
  `@stellar/stellar-sdk`, covering `publishCommitment`, `getCommitment`,
  `submitEnvelope`, `getEnvelope`, and `verifyProof`. 5 tests, all against
  live testnet — real writes, real reads, real rejection paths.
- Rust SDK (`sdk/rust`): real implementation against the `soroban-client`
  crate, covering the same four functions. 4 integration tests, all against
  live testnet.
- CI, issue templates, and contribution guidelines.
