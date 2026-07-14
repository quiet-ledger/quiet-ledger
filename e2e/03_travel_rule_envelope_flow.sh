#!/usr/bin/env bash
# e2e/03_travel_rule_envelope_flow.sh
#
# Exercises the travel_rule_envelope contract: submit -> read back -> confirm
# a resubmission of the same tx_ref is rejected. Idempotent: each run derives
# a fresh tx_ref from a counter file in .state/, so re-running never collides
# with a previous run's already-submitted envelope.

source "$(dirname "$0")/lib.sh"
require_cmd stellar

COUNTER_FILE="${STATE_DIR}/${NETWORK}.envelope_nonce"
NONCE=0
[ -f "${COUNTER_FILE}" ] && NONCE="$(cat "${COUNTER_FILE}")"
NONCE=$((NONCE + 1))
echo "${NONCE}" > "${COUNTER_FILE}"

# 32-byte tx_ref derived from the nonce, zero-padded.
TX_REF="$(printf '%064x' "${NONCE}")"
PAYLOAD_HASH="ab00000000000000000000000000000000000000000000000000000000000000"
PAYLOAD_HASH="${PAYLOAD_HASH:0:64}"

log "anchor A submits an envelope for tx_ref ${TX_REF}"
invoke travel_rule_envelope "${ANCHOR_A}" -- submit_envelope \
  --tx_ref "${TX_REF}" \
  --sender "$(addr "${ANCHOR_A}")" \
  --recipient "$(addr "${ANCHOR_B}")" \
  --payload_hash "${PAYLOAD_HASH}" >/dev/null

STORED_HASH="$(invoke travel_rule_envelope "${ADMIN}" -- get_envelope --tx_ref "${TX_REF}" | jq -r '.payload_hash // empty')"
[ "${STORED_HASH}" = "${PAYLOAD_HASH}" ] && ok "get_envelope returns the submitted hash" || \
  die "expected payload_hash ${PAYLOAD_HASH}, got ${STORED_HASH}"

log "resubmitting the same tx_ref should be rejected"
if invoke travel_rule_envelope "${ANCHOR_A}" -- submit_envelope \
  --tx_ref "${TX_REF}" \
  --sender "$(addr "${ANCHOR_A}")" \
  --recipient "$(addr "${ANCHOR_B}")" \
  --payload_hash "${PAYLOAD_HASH}" >/dev/null 2>&1; then
  die "resubmission of an existing tx_ref unexpectedly succeeded"
else
  ok "resubmission correctly rejected (EnvelopeAlreadySubmitted)"
fi

ok "travel_rule_envelope flow complete"
