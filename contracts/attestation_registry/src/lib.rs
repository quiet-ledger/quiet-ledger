#![no_std]

//! Stores each anchor's current commitment (e.g. the Merkle root of its
//! cleared-user set) and a rotation history. An anchor may only update its
//! own commitment (`require_auth`) — there is no shared admin role here, so
//! this contract has no analogous access-control gap to `proof_verifier`'s
//! verifying-key registration.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Commitment {
    pub root: BytesN<32>,
    pub published_at_ledger: u32,
}

#[contracttype]
enum DataKey {
    Commitment(Address),
    History(Address),
}

#[contract]
pub struct AttestationRegistryContract;

#[contractimpl]
impl AttestationRegistryContract {
    /// Publishes a new commitment for `anchor`, moving the previous
    /// commitment (if any) into that anchor's history.
    pub fn publish_commitment(env: Env, anchor: Address, root: BytesN<32>) {
        anchor.require_auth();

        let commitment = Commitment {
            root,
            published_at_ledger: env.ledger().sequence(),
        };

        if let Some(previous) = env
            .storage()
            .persistent()
            .get::<_, Commitment>(&DataKey::Commitment(anchor.clone()))
        {
            let mut history: Vec<Commitment> = env
                .storage()
                .persistent()
                .get(&DataKey::History(anchor.clone()))
                .unwrap_or(vec![&env]);
            history.push_back(previous);
            env.storage()
                .persistent()
                .set(&DataKey::History(anchor.clone()), &history);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Commitment(anchor), &commitment);
    }

    pub fn get_commitment(env: Env, anchor: Address) -> Option<Commitment> {
        env.storage().persistent().get(&DataKey::Commitment(anchor))
    }

    pub fn get_history(env: Env, anchor: Address) -> Vec<Commitment> {
        env.storage()
            .persistent()
            .get(&DataKey::History(anchor))
            .unwrap_or(vec![&env])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn client() -> (Env, AttestationRegistryContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(AttestationRegistryContract, ());
        let client = AttestationRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn root(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    #[test]
    fn publish_then_get_commitment_round_trips() {
        let (env, client) = client();
        let anchor = Address::generate(&env);
        env.ledger().with_mut(|li| li.sequence_number = 42);

        client.publish_commitment(&anchor, &root(&env, 1));

        let stored = client.get_commitment(&anchor).unwrap();
        assert_eq!(stored.root, root(&env, 1));
        assert_eq!(stored.published_at_ledger, 42);
    }

    #[test]
    fn republishing_moves_previous_commitment_into_history() {
        let (env, client) = client();
        let anchor = Address::generate(&env);

        client.publish_commitment(&anchor, &root(&env, 1));
        client.publish_commitment(&anchor, &root(&env, 2));

        let current = client.get_commitment(&anchor).unwrap();
        assert_eq!(current.root, root(&env, 2));

        let history = client.get_history(&anchor);
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().root, root(&env, 1));
    }

    #[test]
    fn get_commitment_returns_none_for_unknown_anchor() {
        let (env, client) = client();
        let anchor = Address::generate(&env);
        assert!(client.get_commitment(&anchor).is_none());
    }

    #[test]
    fn get_history_returns_empty_vec_for_anchor_with_one_commitment() {
        let (env, client) = client();
        let anchor = Address::generate(&env);
        client.publish_commitment(&anchor, &root(&env, 1));
        assert_eq!(client.get_history(&anchor).len(), 0);
    }
}
