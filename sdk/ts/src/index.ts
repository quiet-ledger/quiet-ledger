/**
 * Quiet Ledger TypeScript SDK — real implementation against
 * @stellar/stellar-sdk's RPC client, wired to the contracts actually
 * deployed and tracked in `e2e/.state/` (see README for current addresses).
 */

import {
  Address,
  BASE_FEE,
  Contract,
  Keypair,
  rpc,
  scValToNative,
  TransactionBuilder,
  xdr,
} from "@stellar/stellar-sdk";

export interface QuietLedgerConfig {
  networkPassphrase: string;
  rpcUrl: string;
  attestationRegistryContractId: string;
  proofVerifierContractId: string;
  travelRuleEnvelopeContractId: string;
}

export interface Groth16Proof {
  proofA: Buffer; // 64 bytes
  proofB: Buffer; // 128 bytes
  proofC: Buffer; // 64 bytes
  publicInputs: Buffer[]; // each 32 bytes; last entry is the expiry ledger
}

async function simulateRead<T>(
  config: Pick<QuietLedgerConfig, "rpcUrl" | "networkPassphrase">,
  contractId: string,
  method: string,
  args: xdr.ScVal[],
  sourceAccountId: string,
): Promise<T> {
  const server = new rpc.Server(config.rpcUrl);
  const contract = new Contract(contractId);
  const sourceAccount = await server.getAccount(sourceAccountId);

  const tx = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: config.networkPassphrase,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(`simulation failed: ${sim.error}`);
  }
  if (!sim.result) {
    throw new Error("simulation returned no result");
  }
  return scValToNative(sim.result.retval) as T;
}

async function submitWrite<T>(
  config: Pick<QuietLedgerConfig, "rpcUrl" | "networkPassphrase">,
  contractId: string,
  method: string,
  args: xdr.ScVal[],
  signerSecret: string,
): Promise<T> {
  const server = new rpc.Server(config.rpcUrl);
  const contract = new Contract(contractId);
  const keypair = Keypair.fromSecret(signerSecret);
  const sourceAccount = await server.getAccount(keypair.publicKey());

  const tx = new TransactionBuilder(sourceAccount, {
    fee: BASE_FEE,
    networkPassphrase: config.networkPassphrase,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  prepared.sign(keypair);

  const sendResult = await server.sendTransaction(prepared);
  if (sendResult.status === "ERROR") {
    throw new Error(`transaction submission failed: ${JSON.stringify(sendResult.errorResult)}`);
  }

  let getResult = await server.getTransaction(sendResult.hash);
  const start = Date.now();
  while (getResult.status === rpc.Api.GetTransactionStatus.NOT_FOUND) {
    if (Date.now() - start > 30_000) {
      throw new Error(`timed out waiting for transaction ${sendResult.hash}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 1500));
    getResult = await server.getTransaction(sendResult.hash);
  }

  if (getResult.status !== rpc.Api.GetTransactionStatus.SUCCESS) {
    throw new Error(`transaction failed: ${JSON.stringify(getResult)}`);
  }

  if (!getResult.returnValue) {
    return undefined as T;
  }
  return scValToNative(getResult.returnValue) as T;
}

// ── attestation_registry ─────────────────────────────────────────────────────

export interface Commitment {
  root: Buffer;
  published_at_ledger: number;
}

/** Publishes a new commitment for the calling anchor. */
export async function publishCommitment(
  config: QuietLedgerConfig,
  anchorSecret: string,
  root: Buffer,
): Promise<void> {
  const anchor = Keypair.fromSecret(anchorSecret).publicKey();
  await submitWrite(
    config,
    config.attestationRegistryContractId,
    "publish_commitment",
    [Address.fromString(anchor).toScVal(), xdr.ScVal.scvBytes(root)],
    anchorSecret,
  );
}

/** Reads an anchor's current commitment, or null if it has never published one. */
export async function getCommitment(
  config: QuietLedgerConfig,
  anchorAccountId: string,
  readAsAccountId: string,
): Promise<Commitment | null> {
  return simulateRead<Commitment | null>(
    config,
    config.attestationRegistryContractId,
    "get_commitment",
    [Address.fromString(anchorAccountId).toScVal()],
    readAsAccountId,
  );
}

// ── travel_rule_envelope ─────────────────────────────────────────────────────

export interface Envelope {
  sender: string;
  recipient: string;
  payload_hash: Buffer;
  submitted_at_ledger: number;
}

/**
 * Submits a travel-rule envelope: a hash pointer to the encrypted payload
 * exchanged directly with the recipient anchor off-chain. Throws if
 * `txRef` was already submitted (the contract rejects resubmission).
 */
export async function submitEnvelope(
  config: QuietLedgerConfig,
  senderSecret: string,
  txRef: Buffer,
  recipientAccountId: string,
  payloadHash: Buffer,
): Promise<void> {
  const sender = Keypair.fromSecret(senderSecret).publicKey();
  await submitWrite(
    config,
    config.travelRuleEnvelopeContractId,
    "submit_envelope",
    [
      xdr.ScVal.scvBytes(txRef),
      Address.fromString(sender).toScVal(),
      Address.fromString(recipientAccountId).toScVal(),
      xdr.ScVal.scvBytes(payloadHash),
    ],
    senderSecret,
  );
}

/** Reads a travel-rule envelope by transaction reference, or null if unset. */
export async function getEnvelope(
  config: QuietLedgerConfig,
  txRef: Buffer,
  readAsAccountId: string,
): Promise<Envelope | null> {
  return simulateRead<Envelope | null>(
    config,
    config.travelRuleEnvelopeContractId,
    "get_envelope",
    [xdr.ScVal.scvBytes(txRef)],
    readAsAccountId,
  );
}

// ── proof_verifier ───────────────────────────────────────────────────────────

/**
 * Verifies a proof against a registered circuit's verifying key.
 *
 * @remarks As of this writing no circuit has a real trusted-setup verifying
 * key registered on the deployed contract (see the open "Run trusted setup"
 * issue), so calling this for any `circuitId` will correctly return an
 * `UnknownCircuit` contract error rather than a verification result — that
 * failure path itself is real and tested (see `__tests__`), not a stub.
 */
export async function verifyProof(
  config: QuietLedgerConfig,
  circuitId: string,
  proof: Groth16Proof,
  readAsAccountId: string,
): Promise<boolean> {
  const publicInputs = xdr.ScVal.scvVec(
    proof.publicInputs.map((b) => xdr.ScVal.scvBytes(b)),
  );
  return simulateRead<boolean>(
    config,
    config.proofVerifierContractId,
    "verify_proof",
    [
      xdr.ScVal.scvSymbol(circuitId),
      xdr.ScVal.scvBytes(proof.proofA),
      xdr.ScVal.scvBytes(proof.proofB),
      xdr.ScVal.scvBytes(proof.proofC),
      publicInputs,
    ],
    readAsAccountId,
  );
}
