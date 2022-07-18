extern crate ed25519_dalek;

use blake2::digest::Key;
use ed25519_dalek::Keypair;
use ed25519_dalek::Signature;

use rand_core::{OsRng};
use blake2::{Blake2b512,Digest, Blake2bVar};
use blake2::digest::Update;
use blake2::digest::VariableOutput;

use pretty_hex::*;


const HASH_SIZE: usize = 16;

fn generate_voting_round_signing_key() -> Keypair {
    let keypair: Keypair = Keypair::generate(&mut OsRng);
    keypair
}

fn generate_voter_id(voting_round_signing_key: &Keypair) -> [u8; HASH_SIZE] {
    let mut hasher = Blake2bVar::new(HASH_SIZE).unwrap();

    let mut buf = [0u8; HASH_SIZE];

    hasher.update(voting_round_signing_key.public.as_bytes());

    hasher.finalize_variable(&mut buf).unwrap();

    buf
}

fn get_hex(raw_hex: &[u8; HASH_SIZE]) -> String {
    let split = simple_hex(&raw_hex).replace(" ", "");
    split
}



fn main() {
    let signing_key = generate_voting_round_signing_key();
    let voter_id = generate_voter_id(&signing_key);

    println!("Voter ID: {}", get_hex(&voter_id));
}

