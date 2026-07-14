#!/usr/bin/env bash
# e2e/01_setup_and_deploy.sh
#
# Idempotent setup: identities, build all three contract wasms, deploy each
# once and cache its id. Safe to re-run.

source "$(dirname "$0")/lib.sh"
require_cmd stellar

log "network: ${NETWORK}"

for id in "${ADMIN}" "${ANCHOR_A}" "${ANCHOR_B}" "${CO_SIGNER_C}"; do
  ensure_identity "${id}"
done

log "building contract wasms"
( cd "${REPO_ROOT}" && cargo build --target wasm32v1-none --release )

WASM_DIR="${REPO_ROOT}/target/wasm32v1-none/release"
deploy_if_needed attestation_registry "${WASM_DIR}/quiet_ledger_attestation_registry.wasm"
deploy_if_needed proof_verifier "${WASM_DIR}/quiet_ledger_proof_verifier.wasm"
deploy_if_needed travel_rule_envelope "${WASM_DIR}/quiet_ledger_travel_rule_envelope.wasm"
deploy_if_needed threshold_attestation "${WASM_DIR}/quiet_ledger_threshold_attestation.wasm"

TA_INIT_MARKER="${STATE_DIR}/${NETWORK}.threshold_attestation.initialized"
if [ -f "${TA_INIT_MARKER}" ]; then
  ok "threshold_attestation already initialized"
else
  log "initializing threshold_attestation (2-of-3: admin, anchor_a, co_signer_c)"
  invoke threshold_attestation "${ADMIN}" -- initialize \
    --signers "[\"$(addr "${ADMIN}")\",\"$(addr "${ANCHOR_A}")\",\"$(addr "${CO_SIGNER_C}")\"]" \
    --threshold 2 >/dev/null
  touch "${TA_INIT_MARKER}"
  ok "threshold_attestation initialized"
fi

ok "setup complete"
