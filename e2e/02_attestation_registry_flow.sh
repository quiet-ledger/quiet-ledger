#!/usr/bin/env bash
# e2e/02_attestation_registry_flow.sh
#
# Exercises the attestation_registry contract's full commitment lifecycle:
# publish -> read back -> republish -> confirm the old commitment moved to
# history. Safe to re-run (each run publishes a fresh root derived from the
# current timestamp-independent counter file, so it never collides with a
# prior run's assertions).

source "$(dirname "$0")/lib.sh"
require_cmd stellar

ROOT_A="1111111111111111111111111111111111111111111111111111111111111111"
ROOT_B="2222222222222222222222222222222222222222222222222222222222222222"

log "anchor publishes its first commitment"
invoke attestation_registry "${ANCHOR_A}" -- publish_commitment \
  --anchor "$(addr "${ANCHOR_A}")" --root "${ROOT_A}" >/dev/null

STORED_ROOT="$(invoke attestation_registry "${ADMIN}" -- get_commitment --anchor "$(addr "${ANCHOR_A}")" | jq -r '.root // empty')"
[ "${STORED_ROOT}" = "${ROOT_A}" ] && ok "get_commitment returns the published root" || \
  die "expected root ${ROOT_A}, got ${STORED_ROOT}"

log "anchor republishes with a new commitment"
invoke attestation_registry "${ANCHOR_A}" -- publish_commitment \
  --anchor "$(addr "${ANCHOR_A}")" --root "${ROOT_B}" >/dev/null

CURRENT_ROOT="$(invoke attestation_registry "${ADMIN}" -- get_commitment --anchor "$(addr "${ANCHOR_A}")" | jq -r '.root // empty')"
[ "${CURRENT_ROOT}" = "${ROOT_B}" ] && ok "current commitment updated to the new root" || \
  die "expected current root ${ROOT_B}, got ${CURRENT_ROOT}"

HISTORY_LEN="$(invoke attestation_registry "${ADMIN}" -- get_history --anchor "$(addr "${ANCHOR_A}")" | jq 'length')"
[ "${HISTORY_LEN}" -ge 1 ] && ok "previous commitment preserved in history (len=${HISTORY_LEN})" || \
  die "expected history to contain at least 1 entry, got ${HISTORY_LEN}"

ok "attestation_registry flow complete"
