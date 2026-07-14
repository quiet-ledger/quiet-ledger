/**
 * Quiet Ledger TypeScript SDK.
 *
 * This is a hand-written interface skeleton, not yet wired to generated
 * contract bindings — those get generated once the contracts have a real
 * testnet deployment (`stellar contract bindings typescript`, tracked as an
 * open issue). The types here define the shape the rest of this SDK, the
 * `agent/` service, and anchor integrators should code against.
 */

export interface QuietLedgerConfig {
  networkPassphrase: string;
  rpcUrl: string;
  attestationRegistryContractId: string;
  proofVerifierContractId: string;
  travelRuleEnvelopeContractId: string;
}

export interface Groth16Proof {
  proofA: Uint8Array; // 64 bytes
  proofB: Uint8Array; // 128 bytes
  proofC: Uint8Array; // 64 bytes
  publicInputs: Uint8Array[]; // each 32 bytes; last entry is the expiry ledger
}

export interface MerkleMembershipClaim {
  /** The circuit id this proof was generated for, e.g. "merkle" or "range". */
  circuitId: string;
  proof: Groth16Proof;
}

/**
 * Verifies a proof against a registered circuit's verifying key by calling
 * the `proof_verifier` contract.
 *
 * @remarks Not yet implemented — requires generated contract bindings
 * (tracked issue: "Build TypeScript SDK"). This signature is the intended
 * public API other modules and anchor integrators should code against.
 */
export async function verifyProof(
  _config: QuietLedgerConfig,
  _claim: MerkleMembershipClaim,
): Promise<boolean> {
  throw new Error("not implemented — see CONTRIBUTING.md and the seeded issues");
}

/**
 * Publishes a new commitment (e.g. a Merkle root) for the calling anchor to
 * the `attestation_registry` contract.
 *
 * @remarks Not yet implemented — see {@link verifyProof}.
 */
export async function publishCommitment(
  _config: QuietLedgerConfig,
  _anchorSecretKey: string,
  _root: Uint8Array,
): Promise<string> {
  throw new Error("not implemented — see CONTRIBUTING.md and the seeded issues");
}
