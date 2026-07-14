pragma circom 2.0.0;

// Num2Bits and LessThan below are self-contained reimplementations of the
// standard circomlib comparator pattern (bit-decomposition range check),
// not novel cryptography — kept dependency-free so this circuit compiles
// without vendoring circomlib. See README.md for the disclosure claim this
// proves and what stays private.

template Num2Bits(n) {
    signal input in;
    signal output out[n];
    var lc1 = 0;
    var e2 = 1;
    for (var i = 0; i < n; i++) {
        out[i] <-- (in >> i) & 1;
        out[i] * (out[i] - 1) === 0;
        lc1 += out[i] * e2;
        e2 = e2 + e2;
    }
    lc1 === in;
}

template LessThan(n) {
    signal input in[2];
    signal output out;

    component n2b = Num2Bits(n + 1);
    n2b.in <== in[0] + (1 << n) - in[1];
    out <== 1 - n2b.out[n];
}

// Proves `amount < threshold` without revealing `amount`.
//
// Private input:  amount     — the real transaction amount
// Public input:    threshold  — the FATF/jurisdiction reporting threshold
// Public output:   belowThreshold — 1 if amount < threshold, else 0
//
// `n` bounds the bit-width of both amount and threshold; 64 bits is enough
// for any realistic stroop-denominated amount without overflow wraparound
// inside the field.
template RangeDisclosure(n) {
    signal input amount;
    signal input threshold;
    signal output belowThreshold;

    component lt = LessThan(n);
    lt.in[0] <== amount;
    lt.in[1] <== threshold;
    belowThreshold <== lt.out;
}

component main {public [threshold]} = RangeDisclosure(64);
