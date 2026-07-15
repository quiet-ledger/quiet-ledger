import { randomBytes } from "node:crypto";
import { Keypair } from "@stellar/stellar-sdk";
import { describe, expect, it } from "vitest";
import {
  getCommitment,
  getEnvelope,
  publishCommitment,
  submitEnvelope,
  verifyProof,
  type QuietLedgerConfig,
} from "../src/index.js";

// Real, currently-deployed testnet contracts — see the repo README for how
// these were obtained (e2e/.state/ after running e2e/01_setup_and_deploy.sh).
const CONFIG: QuietLedgerConfig = {
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
  attestationRegistryContractId: "CB2RON4OTYIBD7YK7VCRTZIV6NF7PYXDXACC5N2SLDSVNSJ4MHSMXCQT",
  proofVerifierContractId: "CCGSFI3FT3XJH2WI7WAMA7ZY7FA53HT7AAXSP56M3YZ242NQCV5DHQ6F",
  travelRuleEnvelopeContractId: "CADXCVAR45PCU75JJWD2HMVKOJTFBS7RZXENLH5DQHABWTD6HWPFWAX2",
};

// Loaded via `SDK_TEST_ANCHOR_A_SECRET="$(stellar keys secret ql_anchor_a)"`
// in the test-running command — never hardcoded, never logged.
const ANCHOR_A_SECRET = process.env.SDK_TEST_ANCHOR_A_SECRET;
const ANCHOR_A_PUBLIC = "GBPLXSKCYLWIIK6F5ZFZ4ZM65XMWCOWAQZCOIXHQI33ANQPZNMFABRKG";
const ANCHOR_B_PUBLIC = "GARPZ6FFXFT7ZY5S76O3LEGNIUCVWPHEGVL6EC5YVNYQIVUG4OYTUDIX";

const describeIfSecret = ANCHOR_A_SECRET ? describe : describe.skip;

describe("getCommitment / getEnvelope (read-only, live testnet)", () => {
  it("returns null for an anchor that has never published a commitment", async () => {
    // A freshly generated keypair has certainly never published anything.
    const neverPublished = Keypair.random().publicKey();
    const result = await getCommitment(CONFIG, neverPublished, ANCHOR_A_PUBLIC);
    expect(result).toBeNull();
  }, 20_000);

  it("returns null for a tx_ref that was never submitted", async () => {
    const neverSubmitted = randomBytes(32);
    const result = await getEnvelope(CONFIG, neverSubmitted, ANCHOR_A_PUBLIC);
    expect(result).toBeNull();
  }, 20_000);
});

describe("verifyProof (live testnet, no circuit registered yet)", () => {
  it("fails against a circuit id with no registered verifying key", async () => {
    // No trusted setup has been run yet (tracked issue), so this must
    // genuinely fail rather than return a false-positive true/false.
    await expect(
      verifyProof(
        CONFIG,
        "range_disclosure",
        {
          proofA: Buffer.alloc(64),
          proofB: Buffer.alloc(128),
          proofC: Buffer.alloc(64),
          publicInputs: [Buffer.alloc(32), Buffer.alloc(32)],
        },
        ANCHOR_A_PUBLIC,
      ),
    ).rejects.toThrow();
  }, 20_000);
});

describeIfSecret("publishCommitment / submitEnvelope (write path, live testnet)", () => {
  it("publishes a commitment and reads it back", async () => {
    const root = randomBytes(32);
    await publishCommitment(CONFIG, ANCHOR_A_SECRET!, root);

    const stored = await getCommitment(CONFIG, ANCHOR_A_PUBLIC, ANCHOR_A_PUBLIC);
    expect(stored).not.toBeNull();
    expect(stored!.root.equals(root)).toBe(true);
  }, 30_000);

  it("submits an envelope and reads it back, then rejects resubmission", async () => {
    const txRef = randomBytes(32);
    const payloadHash = randomBytes(32);

    await submitEnvelope(CONFIG, ANCHOR_A_SECRET!, txRef, ANCHOR_B_PUBLIC, payloadHash);

    const stored = await getEnvelope(CONFIG, txRef, ANCHOR_A_PUBLIC);
    expect(stored).not.toBeNull();
    expect(stored!.payload_hash.equals(payloadHash)).toBe(true);
    expect(stored!.sender).toBe(ANCHOR_A_PUBLIC);
    expect(stored!.recipient).toBe(ANCHOR_B_PUBLIC);

    await expect(
      submitEnvelope(CONFIG, ANCHOR_A_SECRET!, txRef, ANCHOR_B_PUBLIC, randomBytes(32)),
    ).rejects.toThrow();
  }, 30_000);
});
