//! Quiet Ledger Rust SDK — real implementation against `soroban-client`,
//! wired to the contracts actually deployed and tracked in `e2e/.state/`
//! (see the repo README for current addresses).

use soroban_client::{
    address::{Address, AddressTrait},
    contract::{ContractBehavior, Contracts},
    keypair::{Keypair, KeypairBehavior},
    network::{NetworkPassphrase, Networks},
    soroban_rpc::TransactionStatus,
    transaction::{TransactionBehavior, TransactionBuilder, TransactionBuilderBehavior},
    xdr, Options, Server,
};
use std::fmt;
use std::time::Duration;

pub struct QuietLedgerConfig {
    pub rpc_url: String,
    pub attestation_registry_contract_id: String,
    pub travel_rule_envelope_contract_id: String,
    pub proof_verifier_contract_id: String,
}

#[derive(Debug)]
pub enum SdkError {
    Rpc(String),
    Simulation(String),
    TransactionFailed(String),
    UnexpectedReturnValue(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::Rpc(msg) => write!(f, "rpc error: {msg}"),
            SdkError::Simulation(msg) => write!(f, "simulation error: {msg}"),
            SdkError::TransactionFailed(msg) => write!(f, "transaction failed: {msg}"),
            SdkError::UnexpectedReturnValue(msg) => write!(f, "unexpected return value: {msg}"),
        }
    }
}

impl std::error::Error for SdkError {}

fn server(config: &QuietLedgerConfig) -> Result<Server, SdkError> {
    Server::new(&config.rpc_url, Options::default()).map_err(|e| SdkError::Rpc(format!("{e:?}")))
}

fn address_sc_val(public_key: &str) -> Result<xdr::ScVal, SdkError> {
    Address::new(public_key)
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))?
        .to_sc_val()
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))
}

fn bytes_sc_val(bytes: &[u8]) -> Result<xdr::ScVal, SdkError> {
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| SdkError::UnexpectedReturnValue("expected exactly 32 bytes".into()))?;
    Ok(xdr::ScVal::Bytes(
        arr.to_vec()
            .try_into()
            .map(xdr::ScBytes)
            .map_err(|_| SdkError::UnexpectedReturnValue("could not encode bytes".into()))?,
    ))
}

/// Looks up a field by symbol name in a `#[contracttype]` struct's ScVal::Map
/// representation.
fn map_get<'a>(scval: &'a xdr::ScVal, key: &str) -> Option<&'a xdr::ScVal> {
    if let xdr::ScVal::Map(Some(map)) = scval {
        for entry in map.0.iter() {
            if let xdr::ScVal::Symbol(sym) = &entry.key {
                if sym.to_utf8_string().as_deref() == Ok(key) {
                    return Some(&entry.val);
                }
            }
        }
    }
    None
}

fn map_get_bytes(scval: &xdr::ScVal, key: &str) -> Result<Vec<u8>, SdkError> {
    match map_get(scval, key) {
        Some(xdr::ScVal::Bytes(b)) => Ok(b.0.to_vec()),
        other => Err(SdkError::UnexpectedReturnValue(format!(
            "expected bytes for '{key}', got {other:?}"
        ))),
    }
}

fn map_get_u32(scval: &xdr::ScVal, key: &str) -> Result<u32, SdkError> {
    match map_get(scval, key) {
        Some(xdr::ScVal::U32(n)) => Ok(*n),
        other => Err(SdkError::UnexpectedReturnValue(format!(
            "expected u32 for '{key}', got {other:?}"
        ))),
    }
}

fn map_get_address(scval: &xdr::ScVal, key: &str) -> Result<String, SdkError> {
    match map_get(scval, key) {
        Some(xdr::ScVal::Address(addr)) => Address::from_sc_address(addr)
            .map(|a| a.to_string())
            .map_err(|e| SdkError::UnexpectedReturnValue(format!("{e:?}"))),
        other => Err(SdkError::UnexpectedReturnValue(format!(
            "expected address for '{key}', got {other:?}"
        ))),
    }
}

async fn simulate_read(
    config: &QuietLedgerConfig,
    contract_id: &str,
    method: &str,
    args: Vec<xdr::ScVal>,
    read_as_public_key: &str,
) -> Result<xdr::ScVal, SdkError> {
    let rpc = server(config)?;
    let mut account = rpc
        .get_account(read_as_public_key)
        .await
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))?;
    let contract =
        Contracts::new(contract_id).map_err(|e| SdkError::Rpc(format!("contract id: {e:?}")))?;

    let tx = TransactionBuilder::new(&mut account, Networks::testnet(), None)
        .fee(1_000u32)
        .add_operation(contract.call(method, Some(args)))
        .build_for_simulation();

    let sim = rpc
        .simulate_transaction(&tx, None)
        .await
        .map_err(|e| SdkError::Simulation(format!("{e:?}")))?;

    match sim.to_result() {
        Some((scval, _auth)) => Ok(scval),
        None => Err(SdkError::Simulation(
            "simulation produced no result (see error field)".into(),
        )),
    }
}

async fn submit_write(
    config: &QuietLedgerConfig,
    contract_id: &str,
    method: &str,
    args: Vec<xdr::ScVal>,
    signer_secret: &str,
) -> Result<Option<xdr::ScVal>, SdkError> {
    let rpc = server(config)?;
    let signer =
        Keypair::from_secret(signer_secret).map_err(|e| SdkError::Rpc(format!("{e:?}")))?;
    let mut account = rpc
        .get_account(&signer.public_key())
        .await
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))?;
    let contract =
        Contracts::new(contract_id).map_err(|e| SdkError::Rpc(format!("contract id: {e:?}")))?;

    let tx = TransactionBuilder::new(&mut account, Networks::testnet(), None)
        .fee(1_000u32)
        .add_operation(contract.call(method, Some(args)))
        .build();

    let mut prepared = rpc
        .prepare_transaction(&tx)
        .await
        .map_err(|e| SdkError::Simulation(format!("{e:?}")))?;
    prepared.sign(&[signer]);

    let response = rpc
        .send_transaction(prepared)
        .await
        .map_err(|e| SdkError::TransactionFailed(format!("{e:?}")))?;

    let result = rpc
        .wait_transaction(&response.hash, Duration::from_secs(30))
        .await
        .map_err(|e| SdkError::TransactionFailed(format!("{e:?}")))?;

    if result.status != TransactionStatus::Success {
        return Err(SdkError::TransactionFailed(format!("{:?}", result.status)));
    }

    let (_meta, ret_val) = result
        .to_result_meta()
        .ok_or_else(|| SdkError::UnexpectedReturnValue("no result meta".into()))?;
    Ok(ret_val)
}

// ── attestation_registry ─────────────────────────────────────────────────────

pub struct Commitment {
    pub root: Vec<u8>,
    pub published_at_ledger: u32,
}

pub async fn get_commitment(
    config: &QuietLedgerConfig,
    anchor_public_key: &str,
    read_as_public_key: &str,
) -> Result<Option<Commitment>, SdkError> {
    let arg = address_sc_val(anchor_public_key)?;
    let scval = simulate_read(
        config,
        &config.attestation_registry_contract_id,
        "get_commitment",
        vec![arg],
        read_as_public_key,
    )
    .await?;

    if matches!(scval, xdr::ScVal::Void) {
        return Ok(None);
    }
    Ok(Some(Commitment {
        root: map_get_bytes(&scval, "root")?,
        published_at_ledger: map_get_u32(&scval, "published_at_ledger")?,
    }))
}

/// Publishes a new commitment for the calling anchor.
pub async fn publish_commitment(
    config: &QuietLedgerConfig,
    anchor_secret: &str,
    root: &[u8],
) -> Result<(), SdkError> {
    let anchor_public_key = Keypair::from_secret(anchor_secret)
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))?
        .public_key();
    let args = vec![address_sc_val(&anchor_public_key)?, bytes_sc_val(root)?];
    submit_write(
        config,
        &config.attestation_registry_contract_id,
        "publish_commitment",
        args,
        anchor_secret,
    )
    .await?;
    Ok(())
}

// ── travel_rule_envelope ─────────────────────────────────────────────────────

pub struct Envelope {
    pub sender: String,
    pub recipient: String,
    pub payload_hash: Vec<u8>,
    pub submitted_at_ledger: u32,
}

pub async fn get_envelope(
    config: &QuietLedgerConfig,
    tx_ref: &[u8],
    read_as_public_key: &str,
) -> Result<Option<Envelope>, SdkError> {
    let arg = bytes_sc_val(tx_ref)?;
    let scval = simulate_read(
        config,
        &config.travel_rule_envelope_contract_id,
        "get_envelope",
        vec![arg],
        read_as_public_key,
    )
    .await?;

    if matches!(scval, xdr::ScVal::Void) {
        return Ok(None);
    }
    Ok(Some(Envelope {
        sender: map_get_address(&scval, "sender")?,
        recipient: map_get_address(&scval, "recipient")?,
        payload_hash: map_get_bytes(&scval, "payload_hash")?,
        submitted_at_ledger: map_get_u32(&scval, "submitted_at_ledger")?,
    }))
}

/// Submits a travel-rule envelope. Fails if `tx_ref` was already submitted.
pub async fn submit_envelope(
    config: &QuietLedgerConfig,
    sender_secret: &str,
    tx_ref: &[u8],
    recipient_public_key: &str,
    payload_hash: &[u8],
) -> Result<(), SdkError> {
    let sender_public_key = Keypair::from_secret(sender_secret)
        .map_err(|e| SdkError::Rpc(format!("{e:?}")))?
        .public_key();
    let args = vec![
        bytes_sc_val(tx_ref)?,
        address_sc_val(&sender_public_key)?,
        address_sc_val(recipient_public_key)?,
        bytes_sc_val(payload_hash)?,
    ];
    submit_write(
        config,
        &config.travel_rule_envelope_contract_id,
        "submit_envelope",
        args,
        sender_secret,
    )
    .await?;
    Ok(())
}
