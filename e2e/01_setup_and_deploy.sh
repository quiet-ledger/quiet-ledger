#!/usr/bin/env bash
# e2e/01_setup_and_deploy.sh
#
# Idempotent setup: identities, build all three contract wasms, deploy each
# once and cache its id. Safe to re-run.

source "$(dirname "$0")/lib.sh"
require_cmd stellar

log "network: ${NETWORK}"

for id in "${ADMIN}" "${ANCHOR_A}" "${ANCHOR_B}"; do
  ensure_identity "${id}"
done

log "building contract wasms"
( cd "${REPO_ROOT}" && cargo build --target wasm32v1-none --release )

WASM_DIR="${REPO_ROOT}/target/wasm32v1-none/release"
deploy_if_needed attestation_registry "${WASM_DIR}/quiet_ledger_attestation_registry.wasm"
deploy_if_needed proof_verifier "${WASM_DIR}/quiet_ledger_proof_verifier.wasm"
deploy_if_needed travel_rule_envelope "${WASM_DIR}/quiet_ledger_travel_rule_envelope.wasm"

ok "setup complete"
