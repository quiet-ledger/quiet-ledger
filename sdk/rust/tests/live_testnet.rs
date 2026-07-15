//! Integration tests against real, live Stellar testnet — the same
//! deployed contracts the TypeScript SDK's tests exercise (see
//! sdk/ts/__tests__/index.test.ts and the repo README for addresses).
//!
//! The write-path tests require SDK_TEST_ANCHOR_A_SECRET to be set (loaded
//! inline via `SDK_TEST_ANCHOR_A_SECRET="$(stellar keys secret ql_anchor_a)"`
//! when running `cargo test` — never hardcoded, never printed) and are
//! skipped otherwise.

use quiet_ledger_sdk::{
    get_commitment, get_envelope, publish_commitment, submit_envelope, QuietLedgerConfig,
};
use soroban_client::keypair::{Keypair, KeypairBehavior};

fn config() -> QuietLedgerConfig {
    QuietLedgerConfig {
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
        attestation_registry_contract_id:
            "CB2RON4OTYIBD7YK7VCRTZIV6NF7PYXDXACC5N2SLDSVNSJ4MHSMXCQT".to_string(),
        travel_rule_envelope_contract_id:
            "CADXCVAR45PCU75JJWD2HMVKOJTFBS7RZXENLH5DQHABWTD6HWPFWAX2".to_string(),
        proof_verifier_contract_id: "CCGSFI3FT3XJH2WI7WAMA7ZY7FA53HT7AAXSP56M3YZ242NQCV5DHQ6F"
            .to_string(),
    }
}

const ANCHOR_A_PUBLIC: &str = "GBPLXSKCYLWIIK6F5ZFZ4ZM65XMWCOWAQZCOIXHQI33ANQPZNMFABRKG";
const ANCHOR_B_PUBLIC: &str = "GARPZ6FFXFT7ZY5S76O3LEGNIUCVWPHEGVL6EC5YVNYQIVUG4OYTUDIX";

#[tokio::test]
async fn get_commitment_returns_none_for_an_anchor_that_never_published() {
    let cfg = config();
    let never_published = Keypair::random().unwrap().public_key();
    let result = get_commitment(&cfg, &never_published, ANCHOR_A_PUBLIC)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn get_envelope_returns_none_for_a_tx_ref_never_submitted() {
    let cfg = config();
    let never_submitted = rand::random::<[u8; 32]>();
    let result = get_envelope(&cfg, &never_submitted, ANCHOR_A_PUBLIC)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn publish_commitment_write_path_round_trips() {
    let Ok(secret) = std::env::var("SDK_TEST_ANCHOR_A_SECRET") else {
        eprintln!("skipping: SDK_TEST_ANCHOR_A_SECRET not set");
        return;
    };
    let cfg = config();
    let root = rand::random::<[u8; 32]>();

    publish_commitment(&cfg, &secret, &root).await.unwrap();

    let stored = get_commitment(&cfg, ANCHOR_A_PUBLIC, ANCHOR_A_PUBLIC)
        .await
        .unwrap()
        .expect("commitment should now exist");
    assert_eq!(stored.root, root.to_vec());
}

#[tokio::test]
async fn submit_envelope_write_path_round_trips_and_rejects_resubmission() {
    let Ok(secret) = std::env::var("SDK_TEST_ANCHOR_A_SECRET") else {
        eprintln!("skipping: SDK_TEST_ANCHOR_A_SECRET not set");
        return;
    };
    let cfg = config();
    let tx_ref = rand::random::<[u8; 32]>();
    let payload_hash = rand::random::<[u8; 32]>();

    submit_envelope(&cfg, &secret, &tx_ref, ANCHOR_B_PUBLIC, &payload_hash)
        .await
        .unwrap();

    let stored = get_envelope(&cfg, &tx_ref, ANCHOR_A_PUBLIC)
        .await
        .unwrap()
        .expect("envelope should now exist");
    assert_eq!(stored.payload_hash, payload_hash.to_vec());
    assert_eq!(stored.sender, ANCHOR_A_PUBLIC);
    assert_eq!(stored.recipient, ANCHOR_B_PUBLIC);

    let resubmit = submit_envelope(
        &cfg,
        &secret,
        &tx_ref,
        ANCHOR_B_PUBLIC,
        &rand::random::<[u8; 32]>(),
    )
    .await;
    assert!(
        resubmit.is_err(),
        "resubmitting the same tx_ref should fail"
    );
}
