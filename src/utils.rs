
use ed25519_dalek::Keypair;

use blake2::digest::Update;
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;

use pretty_hex::simple_hex;

use crate::constants::{HASH_SIZE};
use crate::structs::VoterId;
use rand_core::OsRng;



pub fn generate_voting_round_signing_key() -> Keypair {
  let keypair: Keypair = Keypair::generate(&mut OsRng);
  keypair
}

pub fn generate_voter_id(voting_round_signing_key: &Keypair) -> VoterId {
  let mut hasher = Blake2bVar::new(HASH_SIZE).unwrap();

  let mut buf = [0u8; HASH_SIZE];

  hasher.update(voting_round_signing_key.public.as_bytes());

  hasher.finalize_variable(&mut buf).unwrap();

  buf
}

pub fn get_hex(raw_hex: &VoterId) -> String {
  let split = simple_hex(&raw_hex).replace(" ", "");
  split
}
