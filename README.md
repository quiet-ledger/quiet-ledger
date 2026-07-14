# Quiet Ledger

A zero-knowledge travel-rule protocol for Stellar anchors: prove SEP-31
compliance to a counterpart anchor — sanctions-list clearance, threshold
disclosure, joint institutional attestation — without ever handing over the
underlying PII.

> **Status: pre-alpha, actively being designed.** No circuit or contract here
> is production-ready or audited. See [Roadmap](#roadmap) and the seeded
> issues for what's actually built versus planned.

## Live on testnet

All three contracts are deployed and verified working on Stellar Testnet
(the full `e2e/` suite passes against these, live, not just locally):

| Contract | Address |
|---|---|
| `attestation_registry` | [`CC76EO6XTIDFE6YJUSBZZ7QOWPBLXJCSSEWZI2XQDRWW25RCPPTN7C5G`](https://stellar.expert/explorer/testnet/contract/CC76EO6XTIDFE6YJUSBZZ7QOWPBLXJCSSEWZI2XQDRWW25RCPPTN7C5G) |
| `proof_verifier` | [`CDTNSLU4OU3JJ7GEGCP3V4XIN6OKGNPKDWYU6GEFOKE2IZLPMMB2IHFB`](https://stellar.expert/explorer/testnet/contract/CDTNSLU4OU3JJ7GEGCP3V4XIN6OKGNPKDWYU6GEFOKE2IZLPMMB2IHFB) |
| `travel_rule_envelope` | [`CCPTCVMCVY2IRGOIF4IIT3Q5SZQKPPMASUCLXHE4ZEAWDKLJ4IUNIFYN`](https://stellar.expert/explorer/testnet/contract/CCPTCVMCVY2IRGOIF4IIT3Q5SZQKPPMASUCLXHE4ZEAWDKLJ4IUNIFYN) |

## The problem

Anchors running SEP-31 cross-border payments are required to satisfy FATF
travel-rule obligations and sanctions screening, but there is no shared
open-source protocol for this. The [Stellar Anchor
Platform](https://github.com/stellar/anchor-platform) is explicit that it is
scaffolding only — it "never stores personally-identifiable information...
acts as a proxy server," leaving all compliance logic to each anchor. Mature
travel-rule networks used elsewhere (TRISA, Notabene, Sygna Bridge,
VerifyVASP) show no evidence of Stellar support. The result: anchors either
build bespoke, usually manual PII-sharing flows, or under-comply — both bad
outcomes for a network whose flagship use case is cross-border payments.

## The approach

Instead of anchors exchanging raw PII to prove compliance, Quiet Ledger lets
an anchor prove a *claim about* its counterparty's compliance status via
zero-knowledge proofs:

- **Merkle-membership proof** — "this user is on my cleared/sanctions-checked
  list" without revealing the list or the user's identity to the counterpart.
- **Range-disclosure proof** — "this transaction is under/over the
  travel-rule reporting threshold" without revealing the exact amount.
- **Threshold attestation** — joint sign-off from k-of-n institutional
  co-signers on an attestation, verified on-chain.

Proofs are generated off-chain and verified on-chain via Soroban contracts;
only proof outputs and hashed pointers to the real (encrypted, off-chain) PII
payload ever touch the ledger.

## Repository layout

```
circuits/     Circom circuits (merkle_membership, range_disclosure, threshold_attestation)
contracts/    Soroban contracts (attestation_registry, proof_verifier, travel_rule_envelope)
sdk/          TypeScript and Rust SDKs for anchor integration
agent/        Reference off-chain agent extending the Anchor Platform's SEP-12 server
e2e/          Idempotent stellar-cli scripts exercising the full flow on testnet
docs/         RFC, threat model, architecture notes
```

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for how the pieces fit together and
[`docs/RFC.md`](docs/RFC.md) for the protocol specification in progress.

## What's genuinely reused vs. built from scratch

This project sits in the same GitHub account as
[zksoroban](https://github.com/iamkingvalor/zksoroban), a minimal reference
implementation of the Circom + snarkjs (Groth16) zero-knowledge toolchain on
Soroban. Quiet Ledger reuses two concrete things from that work:

1. The proven **build/rebuild flow** for Circom circuits (see
   `circuits/*/README.md` in this repo, following the same pattern as
   zksoroban's `poseidon_preimage` reference circuit).
2. zksoroban's existing **generic on-chain Groth16 verifier contract**
   (`contracts/verifier`) — this repo's `proof_verifier` contracts extend
   that verifier rather than reimplementing Groth16 verification on Soroban
   from zero, which is the hard part of any ZK-on-chain system.

Everything else — the three circuits' actual constraint logic, the
attestation registry, the travel-rule envelope, the off-chain agent, the
SDKs — is new design work for this project, tracked as issues (see below).
The three circuit names existed as empty compiled build artifacts elsewhere
in this account's history with no committed source; nothing usable was
inherited beyond the toolchain and the verifier contract, and this README
says so plainly rather than overstating reuse.

## Roadmap

| Phase | Focus |
|---|---|
| Months 1–2 | Protocol RFC, threat model, circuit design |
| Months 3–5 | Registry + verifier contracts on testnet, reference agent, first SDK |
| Months 6–8 | Pilot anchor integration, encrypted envelope relay, external audit |
| Months 9–10 | SEP proposal refinement, multi-anchor interop testing |
| Months 11–12 | Mainnet-ready release, SCF Build Award application |

Every item on this roadmap is tracked as a GitHub issue under the matching
milestone — see [Issues](../../issues) and [Milestones](../../milestones).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Issues labeled `good first issue`
are scoped for a first contribution; every issue uses the "Wave task"
template with explicit acceptance criteria and difficulty rating.

## License

[MIT](LICENSE)
