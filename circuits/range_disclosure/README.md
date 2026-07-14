# Range Disclosure Circuit

Proves `amount < threshold` without revealing `amount`. This is the one
circuit in this repository that is fully implemented and verified today —
the other two are design docs pending implementation (see their own
READMEs).

Inputs:

- private: `amount`
- public: `threshold`

Output:

- public: `belowThreshold` — 1 if `amount < threshold`, else 0

Implementation: a self-contained bit-decomposition range check (the
standard `Num2Bits` + `LessThan` pattern also found in circomlib), kept
dependency-free so this circuit compiles without vendoring circomlib.
64-bit width, sufficient for any realistic stroop-denominated amount without
field-overflow wraparound.

## Rebuilding the artifacts

```bash
circom circuit.circom --r1cs --wasm --sym -o build
```

## Generating and checking a witness

```bash
echo '{"amount": "500", "threshold": "1000"}' > input.json
node build/circuit_js/generate_witness.js build/circuit_js/circuit.wasm input.json witness.wtns
npx snarkjs wtns export json witness.wtns witness.json
# witness.json[1] is the belowThreshold output signal (1 in this example)
```

## Verified behavior

Manually checked during development: `amount=500, threshold=1000` →
`belowThreshold=1`; `amount=1500, threshold=1000` → `belowThreshold=0`.

## Important note

No trusted-setup ceremony has been run for this circuit yet — that's a
Months 3–5 milestone (see the repo-level roadmap), tracked as an open issue.
Do not use this for anything beyond local development until a setup exists
and the external audit (Months 6–8) has run.
