extern crate ed25519_dalek;

use core::panic;
use rand::Rng;
use rand::seq::SliceRandom;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use ed25519_dalek::Keypair;

use blake2::digest::Update;
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use rand_core::OsRng;

use pretty_hex::*;

const HASH_SIZE: usize = 16;
const VOTERS: usize = 15;
const COMMITTEE_MEMBERS: usize = 5;
const THRESHOLD: f64 = 50 as f64/100 as f64;

fn generate_voting_round_signing_key() -> Keypair {
    let keypair: Keypair = Keypair::generate(&mut OsRng);
    keypair
}

fn generate_voter_id(voting_round_signing_key: &Keypair) -> VoterId {
    let mut hasher = Blake2bVar::new(HASH_SIZE).unwrap();

    let mut buf = [0u8; HASH_SIZE];

    hasher.update(voting_round_signing_key.public.as_bytes());

    hasher.finalize_variable(&mut buf).unwrap();

    buf
}

fn get_hex(raw_hex: &VoterId) -> String {
    let split = simple_hex(&raw_hex).replace(" ", "");
    split
}

type VoterId = [u8; HASH_SIZE];

#[derive(Debug)]
struct Voter {
    voting_round_signing_key: Keypair,
    voter_id: VoterId,
}

#[derive(Debug, Clone, Copy)]
struct Vote {
    voter_id: VoterId,
    vote: VoteType,
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {:?}", pretty_hex(&self.voter_id), self.vote)
    }
}

#[derive(Debug, Clone, Copy)]
enum VotingStages {
    PreVoting,
    Voting,
    PostVoting,
}

#[derive(Debug, Clone, Copy)]
enum VoteType {
    Yes,
    No
}

#[derive(Debug, Clone, Copy)]
struct VotingOutcome {
    yes_votes: usize,
    no_votes: usize,
}

#[derive(Debug, Clone)]
struct VotingRound {
    seed_for_committee_generation: [u8; HASH_SIZE],
    voters: Vec<VoterId>,
    committee_members: Vec<VoterId>,
    received_votes: Vec<Vote>,
    key_confirmations: Vec<VoterId>,
    stage: VotingStages,
    next_round_seed_secret: [u8; HASH_SIZE],
}

impl VotingRound {
    fn apply_for_voting_round(&mut self, voter_ids: Vec<VoterId>) {
        self.voters = voter_ids.to_vec();
    }

    fn update_voting_stage(&mut self, stage: VotingStages) {
        self.stage = stage;
    }

    fn apply_for_committee_membership(&mut self, voter_ids: Vec<VoterId>) {
        // first check if voter had applied to be a voter
        for voter_id in voter_ids.iter() {
            if self.voters.contains(voter_id) {
                self.committee_members.push(*voter_id);
            } else {
                panic!(
                    "Voter {:#?} tried to join committee but was not a voter",
                    voter_id
                );
            }
        }
    }

    fn vote(&mut self, vote: Vec<Vote>) {
        self.received_votes = vote.to_vec();
    }

    fn tally_votes(&self) -> VotingOutcome {
        let mut yes_votes = 0;
        let mut no_votes = 0;
        for vote in self.received_votes.iter() {
            match vote.vote {
                VoteType::Yes => yes_votes += 1,
                VoteType::No => no_votes += 1,
            }
        }
        VotingOutcome {
            yes_votes,
            no_votes,
        }
        
    }

    fn add_next_round_seed_hash(&mut self, seed_hash: [u8; HASH_SIZE]) {
        self.next_round_seed_secret = seed_hash;
    }

    // this is the mechanism by which each commitee member "reveals" part of the key share
    fn add_key_confirmation(&mut self, committee_member_id: VoterId) {
        self.key_confirmations.push(committee_member_id)
    }

    fn generate_next_round_voting_seed(&self) -> Result<[u8; HASH_SIZE], &'static str> {
        // only if the self.key_confirmations.length >= 2/3 of COMMITTEE_MEMBERS, then allow, else panic
        if (self.key_confirmations.len() as f64) < THRESHOLD * (self.committee_members.len() as f64) {
            return Err("Cannot generate next round voting seed if threshold is not met");
        } 
        Ok(self.next_round_seed_secret)
    }   
}

impl Display for Voter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Voter: {} {}",
            get_hex(&self.voter_id),
            simple_hex(self.voting_round_signing_key.public.as_bytes())
        )
    }
}

impl Display for VotingRound {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Voting Round:\nSeed for committee generation:{}\nVoters:{:?}\nCommittee members:{:?}\nReceived votes:{:?}\nStage:{:?}\nNext round seed secret:{}",
            get_hex(&self.seed_for_committee_generation),
            self.voters.iter().map(|v| get_hex(v)).collect::<Vec<String>>(),
            self.committee_members.iter().map(|v| get_hex(&v)).collect::<Vec<String>>(),
            // loop over the received votes, and get_hex the voter_id, and show what they voted for
            self.received_votes.iter().map(|v| format!("{} voted {:?}", get_hex(&v.voter_id), v.vote)).collect::<Vec<String>>(),
            self.stage,
            get_hex(&self.seed_for_committee_generation)
        )
    }
}

fn get_random_vote_type() -> VoteType {
    let mut rng = rand::thread_rng();
    let vote_type: VoteType = if rng.gen() {
        VoteType::Yes
    } else {
        VoteType::No
    };
    vote_type
}

fn main() {
    println!("Creating Voters....");
    // create x voters
    let mut voters: Vec<Voter> = Vec::new();
    for _ in 0..VOTERS {
        let voting_round_signing_key = generate_voting_round_signing_key();
        let voter_id = generate_voter_id(&voting_round_signing_key);
        voters.push(Voter {
            voting_round_signing_key,
            voter_id,
        });
    }
    println!("Voters created!");
    println!("Creating Voting Round....");

    // create a voting round
    let mut first_voting_round = VotingRound {
        // let first voting round have 0 as the seed
        seed_for_committee_generation: [0u8; HASH_SIZE],
        committee_members: Vec::new(),
        voters: Vec::new(),
        received_votes: Vec::new(),
        stage: VotingStages::PreVoting,
        key_confirmations: Vec::new(),
        // keep 0 for now
        next_round_seed_secret: [0u8; HASH_SIZE],
    };

    println!("Voting Round created!");
    println!("Creating Committee....");
    let mut committee_members = Vec::new();
    // choose 5 committee members at random
    for _ in 0..COMMITTEE_MEMBERS {
        let committee_member = voters.choose(&mut OsRng).unwrap();
        committee_members.push(committee_member);
    }

    println!("Committee created!");

    // collect voter ids into a vec
    let mut voter_ids: Vec<VoterId> = Vec::new();
    for voter in voters.iter() {
        voter_ids.push(voter.voter_id);
    }

    println!("Voters applying for voting round...");
    // apply for voting round for each voter
    first_voting_round.apply_for_voting_round(voter_ids);

    println!("Voters applying for committee membership...");

    // create a random list of committee members, and collect their ids into a vec
    let mut committee_member_ids: Vec<VoterId> = Vec::new();
    for committee_member in committee_members.iter() {
        committee_member_ids.push(committee_member.voter_id);
    }
    // apply for committee membership for each committee member
    first_voting_round.apply_for_committee_membership(committee_member_ids.to_vec());

    println!("Pre-Voting Stage completed");
    println!("Voting round starting...");

    // update voting stage to voting
    first_voting_round.update_voting_stage(VotingStages::Voting);

    // make all voters vote
    let mut votes: Vec<Vote> = Vec::new();
    for voter in voters.iter() {
        let vote = Vote {
            voter_id: voter.voter_id,
            vote: get_random_vote_type(),
        };
        votes.push(vote);
    }

    first_voting_round.vote(votes);

    println!("{}", first_voting_round);

    println!("Voting round completed!");

    first_voting_round.update_voting_stage(VotingStages::PostVoting);

    println!("Post-Voting round intitated");

    let tallied_votes = first_voting_round.tally_votes();

    println!("Tallied votes: {:?}", tallied_votes);

    println!("Generating next round seed secret...");
    
    // committee members contribute a part of a random number each, and donate a slice of it as per the number of commitee members
    let mut committee_generated_seeds: Vec<[u8; HASH_SIZE]> = Vec::new();
    for _ in committee_member_ids.to_vec() {
        let mut rng = rand::thread_rng();
        let mut rng_bytes: [u8; HASH_SIZE] = [0u8; HASH_SIZE];
        rng.fill(&mut rng_bytes);
        committee_generated_seeds.push(rng_bytes);
    }

    // take the committee generated seeds, and combine them into a single seed
    let mut combined_committee_generated_seeds: [u8; HASH_SIZE] = [0u8; HASH_SIZE];
    for committee_generated_seed in committee_generated_seeds.iter() {
        for i in 0..HASH_SIZE {
            combined_committee_generated_seeds[i] ^= committee_generated_seed[i];
        }
    }

    let mut round = first_voting_round.add_next_round_seed_hash(combined_committee_generated_seeds);

    println!("Next round seed secret generated:{}", get_hex(&first_voting_round.next_round_seed_secret));

    // adding only 2 committee members confirmations

    println!("Adding key confirmations...");

    first_voting_round.add_key_confirmation(committee_member_ids[0]);
    first_voting_round.add_key_confirmation(committee_member_ids[1]);


    println!("Added key confirmations");
    match first_voting_round.generate_next_round_voting_seed() {
        Ok(voting_seed) => println!("Voting seed: {:?}", voting_seed),
        Err(e) => eprintln!("Error generating next round seed: {:?}", e)
    };

    println!("Adding one more key confirmation");
    first_voting_round.add_key_confirmation(committee_member_ids[2]);
    match first_voting_round.generate_next_round_voting_seed() {
        Ok(voting_seed) => println!("Voting seed: {}", get_hex(&voting_seed)),
        Err(e) => eprintln!("Error generating next round seed: {:?}", e)
    }

}
