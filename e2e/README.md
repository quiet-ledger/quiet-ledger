# End-to-end tests (real Soroban testnet)

Idempotent bash scripts, following the same pattern used elsewhere in this
account's Soroban projects, that deploy the contracts and drive real flows
against a live network (Testnet by default), asserting on-chain state.

## Prerequisites

`stellar` CLI, `jq`, a Rust toolchain with the `wasm32-unknown-unknown`
target. No pre-funded account needed — identities are created and funded via
Friendbot.

## Usage

```bash
cd e2e && ./run_all.sh
# or individually, after setup:
./01_setup_and_deploy.sh
./02_attestation_registry_flow.sh
```

## Coverage

- `01_setup_and_deploy.sh` — identities, build, deploy all three contracts
  (cached, idempotent).
- `02_attestation_registry_flow.sh` — publish → read back → republish →
  confirm the prior commitment moved to history.
- `03_travel_rule_envelope_flow.sh` — submit → read back → confirm a
  resubmission of the same tx_ref is rejected.

## Not yet covered

`proof_verifier` has no e2e script yet: doing so meaningfully requires a
real trusted-setup verifying key from an actual compiled circuit, which
doesn't exist yet (see `circuits/range_disclosure` — compiled but no setup
ceremony run; `merkle_membership` and `threshold_attestation` — design only).
Faking a verifying key here would exercise the contract's plumbing but not
prove anything about real proof verification, so it's better tracked as an
open issue (blocked on the Months 3–5 circuit/setup milestones) than
shipped as a hollow e2e check.
