#[cfg(test)]
mod tests {
    use cryptography_utils::BigInt;
    use cryptography_utils::EC;
    use protocols::two_party_ecdsa::lindell_2017::*;

    #[test]
    fn test_d_log_proof_party_two_party_one() {
        let ec_context = EC::new();
        let party_one_first_message = party_one::KeyGenFirstMsg::create_commitments(&ec_context);
        let party_two_first_message = party_two::KeyGenFirstMsg::create(&ec_context);
        let party_one_second_message = party_one::KeyGenSecondMsg::verify_and_decommit(
            &ec_context,
            &party_one_first_message,
            &party_two_first_message.d_log_proof,
        );
        assert!(party_one_second_message.d_log_proof_result.is_ok());

        let party_two_second_message =
            party_two::KeyGenSecondMsg::verify_commitments_and_dlog_proof(
                &ec_context,
                &party_one_first_message,
                &party_one_second_message,
            );
        assert!(party_two_second_message.d_log_proof_result.is_ok());
    }

    #[test]
    fn test_two_party_keygen() {
        let ec_context = EC::new();

        // secret share generation
        let party_one_first_message = party_one::KeyGenFirstMsg::create_commitments(&ec_context);
        let party_two_first_message = party_two::KeyGenFirstMsg::create(&ec_context);
        let party_one_second_message = party_one::KeyGenSecondMsg::verify_and_decommit(
            &ec_context,
            &party_one_first_message,
            &party_two_first_message.d_log_proof,
        );
        assert!(party_one_second_message.d_log_proof_result.is_ok());

        let party_two_second_message =
            party_two::KeyGenSecondMsg::verify_commitments_and_dlog_proof(
                &ec_context,
                &party_one_first_message,
                &party_one_second_message,
            );
        assert!(party_two_second_message.d_log_proof_result.is_ok());

        // init paillier keypair:
        let paillier_key_pair = party_one::PaillierKeyPair::generate_keypair_and_encrypted_share(
            &party_one_first_message,
        );

        let party_two_paillier = party_two::PaillierPublic {
            ek: paillier_key_pair.ek.clone(),
            encrypted_secret_share: paillier_key_pair.encrypted_share.clone(),
        };

        // zk proof of correct paillier key
        let (challenge, verification_aid) =
            party_two::PaillierPublic::generate_correct_key_challenge(&party_two_paillier);
        let proof_result =
            party_one::PaillierKeyPair::generate_proof_correct_key(&paillier_key_pair, &challenge);
        assert!(proof_result.is_ok());

        let result = party_two::PaillierPublic::verify_correct_key(
            &proof_result.unwrap(),
            &verification_aid,
        );
        assert!(result.is_ok());

        // zk range proof
        let (encrypted_pairs, challenge, proof) = party_one::PaillierKeyPair::generate_range_proof(
            &paillier_key_pair,
            &party_one_first_message,
        );
        assert!(party_two::PaillierPublic::verify_range_proof(
            &party_two_paillier,
            &challenge,
            &encrypted_pairs,
            &proof
        ));
    }

    #[test]
    fn test_two_party_sign() {
        let ec_context = EC::new();
        // assume party1 and party2 engaged with KeyGen in the past resulting in
        // party1 owning private share and paillier key-pair
        // party2 owning private share and paillier encryption of party1 share
        let party_one_private_share_gen =
            party_one::KeyGenFirstMsg::create_commitments(&ec_context);
        let party_two_private_share_gen = party_two::KeyGenFirstMsg::create(&ec_context);

        let keypair = party_one::PaillierKeyPair::generate_keypair_and_encrypted_share(
            &party_one_private_share_gen,
        );

        // creating the ephemeral private shares:

        let party_one_first_message = party_one::KeyGenFirstMsg::create_commitments(&ec_context);
        let party_two_first_message = party_two::KeyGenFirstMsg::create(&ec_context);
        let party_one_second_message = party_one::KeyGenSecondMsg::verify_and_decommit(
            &ec_context,
            &party_one_first_message,
            &party_two_first_message.d_log_proof,
        );
        assert!(party_one_second_message.d_log_proof_result.is_ok());

        let party_two_proof_result = party_two::KeyGenSecondMsg::verify_commitments_and_dlog_proof(
            &ec_context,
            &party_one_first_message,
            &party_one_second_message,
        );
        assert!(party_two_proof_result.d_log_proof_result.is_ok());

        let message = BigInt::from(1234);
        let partial_sig = party_two::PartialSig::compute(
            &ec_context,
            &keypair.ek,
            &keypair.encrypted_share,
            &party_two_private_share_gen,
            &party_two_first_message,
            &party_one_first_message,
            &message,
        );

        let signature = party_one::Signature::compute(
            &ec_context,
            &keypair,
            &partial_sig,
            &party_one_first_message,
            &party_two_first_message,
        );

        let pubkey = party_one::compute_pubkey(
            &ec_context,
            &party_one_private_share_gen,
            &party_two_private_share_gen,
        );
        assert!(party_one::verify(&ec_context, &signature, &pubkey, &message).is_ok())
    }
}