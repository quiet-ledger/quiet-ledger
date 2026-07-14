#![no_std]

//! Stores only a hash pointer to the actual encrypted travel-rule payload
//! (originator/beneficiary IVMS-101 data), keyed by a transaction reference.
//! The real payload is exchanged directly between anchors off-chain — this
//! contract never sees it, only its hash. Each transaction reference may be
//! submitted exactly once (no silent overwrite of a previously submitted
//! envelope).

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    EnvelopeAlreadySubmitted = 1,
}

#[contracttype]
#[derive(Clone)]
pub struct Envelope {
    pub sender: Address,
    pub recipient: Address,
    pub payload_hash: BytesN<32>,
    pub submitted_at_ledger: u32,
}

#[contracttype]
enum DataKey {
    Envelope(BytesN<32>),
}

#[contract]
pub struct TravelRuleEnvelopeContract;

#[contractimpl]
impl TravelRuleEnvelopeContract {
    pub fn submit_envelope(
        env: Env,
        tx_ref: BytesN<32>,
        sender: Address,
        recipient: Address,
        payload_hash: BytesN<32>,
    ) -> Result<(), Error> {
        sender.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::Envelope(tx_ref.clone()))
        {
            return Err(Error::EnvelopeAlreadySubmitted);
        }

        let envelope = Envelope {
            sender,
            recipient,
            payload_hash,
            submitted_at_ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Envelope(tx_ref), &envelope);
        Ok(())
    }

    pub fn get_envelope(env: Env, tx_ref: BytesN<32>) -> Option<Envelope> {
        env.storage().persistent().get(&DataKey::Envelope(tx_ref))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn client() -> (Env, TravelRuleEnvelopeContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TravelRuleEnvelopeContract, ());
        let client = TravelRuleEnvelopeContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn hash(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    #[test]
    fn submit_then_get_envelope_round_trips() {
        let (env, client) = client();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let tx_ref = hash(&env, 1);
        env.ledger().with_mut(|li| li.sequence_number = 7);

        client.submit_envelope(&tx_ref, &sender, &recipient, &hash(&env, 9));

        let stored = client.get_envelope(&tx_ref).unwrap();
        assert_eq!(stored.sender, sender);
        assert_eq!(stored.recipient, recipient);
        assert_eq!(stored.payload_hash, hash(&env, 9));
        assert_eq!(stored.submitted_at_ledger, 7);
    }

    #[test]
    fn resubmitting_the_same_tx_ref_is_rejected() {
        let (env, client) = client();
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let tx_ref = hash(&env, 1);

        client.submit_envelope(&tx_ref, &sender, &recipient, &hash(&env, 9));
        let result = client.try_submit_envelope(&tx_ref, &sender, &recipient, &hash(&env, 10));

        assert_eq!(result, Err(Ok(Error::EnvelopeAlreadySubmitted)));
    }

    #[test]
    fn get_envelope_returns_none_for_unknown_tx_ref() {
        let (env, client) = client();
        assert!(client.get_envelope(&hash(&env, 99)).is_none());
    }
}
