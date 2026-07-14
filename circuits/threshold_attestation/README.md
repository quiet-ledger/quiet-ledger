# Threshold Attestation

**Status: design only, and possibly not a circuit at all.** This directory
exists to keep this account's roadmap language ("threshold attestation")
consistent with the published research, but scaffolding this project
surfaced a real design simplification worth stating plainly.

## The insight

Proving "k-of-n institutional co-signers approved this attestation" does not
inherently require zero-knowledge. Verifying k-of-n signatures over a
message is a standard multisig check, and Soroban can do this natively via
its auth framework — no circuit needed. A circuit is a real requirement only
for the strictly harder variant of *hiding which specific k-of-n signers*
participated (e.g. so an outside observer can't correlate signer identity
with a given attestation). That variant is a different, harder problem and
is explicitly **out of scope** until an issue specifically asks for it — see
`docs/THREAT_MODEL.md`'s "explicitly out of scope for v1" section.

## What to actually build (tracked as an issue)

A k-of-n signature verification check, most naturally as part of
`contracts/attestation_registry` or a dedicated auth module: given n
institutional co-signer public keys and a threshold k, verify that at least
k valid signatures over the attestation payload are present before the
registry accepts a commitment update.

## If the hiding variant is ever needed

That would require a circuit proving "k valid signatures exist among these n
public keys" without revealing which ones — a nontrivial circuit (likely
built on a ring-signature or accumulator construction), not a small addition
to the other two circuits here. Do not attempt this without a dedicated
design doc and review; it is meaningfully harder than `merkle_membership` or
`range_disclosure`.
