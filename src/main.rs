mod constants;
mod structs;
mod utils;

extern crate ed25519_dalek;

use rand::Rng;
use rand::seq::SliceRandom;


use rand_core::OsRng;

use crate::constants::{HASH_SIZE,COMMITTEE_MEMBERS,VOTERS};
use crate::structs::{Vote, VoterId, VotingRound, Voter, VotingStages, get_random_vote_type};
use crate::utils::{generate_voting_round_signing_key, generate_voter_id, get_hex};



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

    // transition voting stage to post-voting stage
    first_voting_round.update_voting_stage(VotingStages::PostVoting);

    println!("Post-Voting round intitated");

    // tally the votes
    let tallied_votes = first_voting_round.tally_votes();

    println!("Tallied votes: {:?}", tallied_votes);
    println!("The vote has {}!", if tallied_votes.yes_votes > tallied_votes.no_votes {
        "passed"
    } else {
        "failed"
    });

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

    first_voting_round.add_next_round_seed_hash(combined_committee_generated_seeds);

    println!("Next round seed secret generated:{}", get_hex(&first_voting_round.next_round_seed_secret));

    // adding only 2 committee members confirmations, this should not allow generation of the seed for the next round

    println!("Adding key confirmations...");

    first_voting_round.add_key_confirmation(committee_member_ids[0]);
    first_voting_round.add_key_confirmation(committee_member_ids[1]);


    println!("Added key confirmations");
    match first_voting_round.generate_next_round_voting_seed() {
        Ok(voting_seed) => println!("Voting seed: {:?}", voting_seed),
        // this branch will be chosen
        Err(e) => eprintln!("Error generating next round seed: {:?}", e)
    };

    println!("Adding one more key confirmation");
    // add the next confirmation, which will allow seed generation
    first_voting_round.add_key_confirmation(committee_member_ids[2]);
    match first_voting_round.generate_next_round_voting_seed() {
        Ok(voting_seed) => println!("Voting seed: {}", get_hex(&voting_seed)),
        Err(e) => eprintln!("Error generating next round seed: {:?}", e)
    }

    println!("Post voting process complete, now the next round can start")
}
