//! Quiet Ledger Rust SDK.
//!
//! Hand-written interface skeleton, not yet wired to a real Soroban RPC
//! client — see `sdk/ts/src/index.ts` for the equivalent TypeScript skeleton
//! and the same caveat. Kept dependency-free for now so this crate builds
//! independently of the contracts workspace's dependency graph.

pub struct QuietLedgerConfig {
    pub network_passphrase: String,
    pub rpc_url: String,
    pub attestation_registry_contract_id: String,
    pub proof_verifier_contract_id: String,
    pub travel_rule_envelope_contract_id: String,
}

pub struct Groth16Proof {
    pub proof_a: [u8; 64],
    pub proof_b: [u8; 128],
    pub proof_c: [u8; 64],
    /// Each entry is 32 bytes; the last entry is the expiry ledger, encoded
    /// the same way as the on-chain contracts expect.
    pub public_inputs: Vec<[u8; 32]>,
}

/// Verifies a proof against a registered circuit's verifying key by calling
/// the `proof_verifier` contract.
///
/// Not yet implemented — requires a real Soroban RPC client integration
/// (tracked issue: "Build Rust SDK").
pub fn verify_proof(
    _config: &QuietLedgerConfig,
    _circuit_id: &str,
    _proof: &Groth16Proof,
) -> Result<bool, String> {
    Err("not implemented — see CONTRIBUTING.md and the seeded issues".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_proof_reports_not_implemented_rather_than_panicking() {
        let config = QuietLedgerConfig {
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            attestation_registry_contract_id: String::new(),
            proof_verifier_contract_id: String::new(),
            travel_rule_envelope_contract_id: String::new(),
        };
        let proof = Groth16Proof {
            proof_a: [0; 64],
            proof_b: [0; 128],
            proof_c: [0; 64],
            public_inputs: vec![],
        };
        assert!(verify_proof(&config, "test", &proof).is_err());
    }
}
