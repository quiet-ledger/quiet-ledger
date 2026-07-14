#![no_std]

//! k-of-n institutional co-signer attestation, implemented as a native
//! Soroban multisig check — not a ZK circuit. See
//! `circuits/threshold_attestation/README.md` for why: verifying k-of-n
//! signatures over a statement doesn't need zero-knowledge, and Soroban's
//! auth framework already does exactly this natively.
//!
//! Each provided co-signer must (a) be a member of the registered signer
//! set, (b) appear at most once, and (c) actually authorize this specific
//! call via `require_auth` — meaning the underlying transaction must carry
//! a valid signature from that address. Meeting the threshold with fewer
//! than `threshold` genuinely-authorizing signers is rejected.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidThreshold = 3,
    UnknownSigner = 4,
    DuplicateSigner = 5,
    ThresholdNotMet = 6,
    AlreadyAttested = 7,
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub signers: Vec<Address>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct AttestationRecord {
    pub co_signers: Vec<Address>,
    pub recorded_at_ledger: u32,
}

#[contracttype]
enum DataKey {
    Config,
    Attestation(BytesN<32>),
}

#[contract]
pub struct ThresholdAttestationContract;

#[contractimpl]
impl ThresholdAttestationContract {
    pub fn initialize(env: Env, signers: Vec<Address>, threshold: u32) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }
        if threshold == 0 || threshold > signers.len() {
            return Err(Error::InvalidThreshold);
        }
        env.storage()
            .instance()
            .set(&DataKey::Config, &Config { signers, threshold });
        Ok(())
    }

    /// Records that `co_signers` (each of whom must actually authorize this
    /// call) jointly attest to `statement_hash` — an opaque hash of
    /// whatever off-chain statement is being attested to. A given
    /// `statement_hash` may only be attested once.
    pub fn submit_attestation(
        env: Env,
        statement_hash: BytesN<32>,
        co_signers: Vec<Address>,
    ) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Attestation(statement_hash.clone()))
        {
            return Err(Error::AlreadyAttested);
        }

        for i in 0..co_signers.len() {
            let signer = co_signers.get(i).unwrap();

            let mut is_registered = false;
            for j in 0..config.signers.len() {
                if config.signers.get(j).unwrap() == signer {
                    is_registered = true;
                    break;
                }
            }
            if !is_registered {
                return Err(Error::UnknownSigner);
            }

            for j in (i + 1)..co_signers.len() {
                if co_signers.get(j).unwrap() == signer {
                    return Err(Error::DuplicateSigner);
                }
            }

            signer.require_auth();
        }

        if co_signers.len() < config.threshold {
            return Err(Error::ThresholdNotMet);
        }

        let record = AttestationRecord {
            co_signers,
            recorded_at_ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Attestation(statement_hash), &record);
        Ok(())
    }

    pub fn get_attestation(env: Env, statement_hash: BytesN<32>) -> Option<AttestationRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Attestation(statement_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec};

    fn client() -> (
        Env,
        Vec<Address>,
        ThresholdAttestationContractClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let signers = vec![
            &env,
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];
        let contract_id = env.register(ThresholdAttestationContract, ());
        let client = ThresholdAttestationContractClient::new(&env, &contract_id);
        client.initialize(&signers, &2); // 2-of-3
        (env, signers, client)
    }

    fn hash(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    #[test]
    fn two_of_three_registered_signers_succeeds() {
        let (env, signers, client) = client();
        let statement = hash(&env, 1);
        let co_signers = vec![&env, signers.get(0).unwrap(), signers.get(1).unwrap()];

        client.submit_attestation(&statement, &co_signers);

        let record = client.get_attestation(&statement).unwrap();
        assert_eq!(record.co_signers.len(), 2);
    }

    #[test]
    fn below_threshold_is_rejected() {
        let (env, signers, client) = client();
        let statement = hash(&env, 1);
        let one_signer = vec![&env, signers.get(0).unwrap()];

        let result = client.try_submit_attestation(&statement, &one_signer);
        assert_eq!(result, Err(Ok(Error::ThresholdNotMet)));
    }

    #[test]
    fn unregistered_signer_is_rejected() {
        let (env, signers, client) = client();
        let statement = hash(&env, 1);
        let outsider = Address::generate(&env);
        let co_signers = vec![&env, signers.get(0).unwrap(), outsider];

        let result = client.try_submit_attestation(&statement, &co_signers);
        assert_eq!(result, Err(Ok(Error::UnknownSigner)));
    }

    #[test]
    fn duplicate_signer_does_not_count_twice() {
        let (env, signers, client) = client();
        let statement = hash(&env, 1);
        let same_signer_twice = vec![&env, signers.get(0).unwrap(), signers.get(0).unwrap()];

        let result = client.try_submit_attestation(&statement, &same_signer_twice);
        assert_eq!(result, Err(Ok(Error::DuplicateSigner)));
    }

    #[test]
    fn resubmitting_the_same_statement_is_rejected() {
        let (env, signers, client) = client();
        let statement = hash(&env, 1);
        let co_signers = vec![&env, signers.get(0).unwrap(), signers.get(1).unwrap()];

        client.submit_attestation(&statement, &co_signers);
        let result = client.try_submit_attestation(&statement, &co_signers);
        assert_eq!(result, Err(Ok(Error::AlreadyAttested)));
    }

    #[test]
    fn initialize_rejects_threshold_greater_than_signer_count() {
        let env = Env::default();
        env.mock_all_auths();
        let signers = vec![&env, Address::generate(&env)];
        let contract_id = env.register(ThresholdAttestationContract, ());
        let client = ThresholdAttestationContractClient::new(&env, &contract_id);

        let result = client.try_initialize(&signers, &2);
        assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
    }
}
