#!/usr/bin/env bash
# e2e/run_all.sh
#
# Runs the full end-to-end suite against the configured network (default:
# testnet) in order. Safe to re-run.

set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

bash "${HERE}/01_setup_and_deploy.sh"
bash "${HERE}/02_attestation_registry_flow.sh"
bash "${HERE}/03_travel_rule_envelope_flow.sh"

printf '\n\033[0;32mAll e2e paths passed.\033[0m\n'
