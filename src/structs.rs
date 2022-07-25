
use pretty_hex::*;
use rand::Rng;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use core::panic;
use ed25519_dalek::Keypair;


use crate::constants::{HASH_SIZE,THRESHOLD};
use crate::get_hex;

pub type VoterId = [u8; HASH_SIZE];

#[derive(Debug)]
pub struct Voter {
    pub voting_round_signing_key: Keypair,
    pub voter_id: VoterId,
}

#[derive(Debug, Clone, Copy)]
pub struct Vote {
    pub voter_id: VoterId,
    pub vote: VoteType,
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {:?}", pretty_hex(&self.voter_id), self.vote)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VotingStages {
    PreVoting,
    Voting,
    PostVoting,
}

#[derive(Debug, Clone, Copy)]
pub enum VoteType {
    Yes,
    No
}

#[derive(Debug, Clone, Copy)]
pub struct VotingOutcome {
    pub yes_votes: usize,
    pub no_votes: usize,
}

#[derive(Debug, Clone)]
pub struct VotingRound {
    pub seed_for_committee_generation: [u8; HASH_SIZE],
    pub voters: Vec<VoterId>,
    pub committee_members: Vec<VoterId>,
    pub received_votes: Vec<Vote>,
    pub key_confirmations: Vec<VoterId>,
    pub stage: VotingStages,
    pub next_round_seed_secret: [u8; HASH_SIZE],
}

impl VotingRound {
    pub fn apply_for_voting_round(&mut self, voter_ids: Vec<VoterId>) {
        self.voters = voter_ids.to_vec();
    }

    pub fn update_voting_stage(&mut self, stage: VotingStages) {
        self.stage = stage;
    }

    pub fn apply_for_committee_membership(&mut self, voter_ids: Vec<VoterId>) {
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

    pub fn vote(&mut self, vote: Vec<Vote>) {
        self.received_votes = vote.to_vec();
    }

    pub fn tally_votes(&self) -> VotingOutcome {
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

    pub fn add_next_round_seed_hash(&mut self, seed_hash: [u8; HASH_SIZE]) {
        self.next_round_seed_secret = seed_hash;
    }

    // this is the mechanism by which each commitee member "reveals" part of the key share
    pub fn add_key_confirmation(&mut self, committee_member_id: VoterId) {
        self.key_confirmations.push(committee_member_id)
    }

    pub fn get_threshold(&self) -> f64 {
      (THRESHOLD * (self.committee_members.len() as f64)).floor()
    }

    pub fn generate_next_round_voting_seed(&self) -> Result<[u8; HASH_SIZE], &'static str> {
        // only if the self.key_confirmations.length >= 2/3 of COMMITTEE_MEMBERS, then allow, else panic
        if (self.key_confirmations.len() as f64) < self.get_threshold() {
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
            "=========================================================================\nVoting Round:\nVoters:{:?}\nCommittee members:{:?}\nReceived votes:{:?}\nStage:{:?}\nNext round seed secret:{}\n============================================================================================",
            self.voters.iter().map(|v| get_hex(v)).collect::<Vec<String>>(),
            self.committee_members.iter().map(|v| get_hex(&v)).collect::<Vec<String>>(),
            // loop over the received votes, and get_hex the voter_id, and show what they voted for in table format
            self.received_votes.iter().map(|v| format!("{} voted {:?}", get_hex(&v.voter_id), v.vote)).collect::<Vec<String>>(),
            self.stage,
            get_hex(&self.seed_for_committee_generation)
        )
    }
}

pub fn get_random_vote_type() -> VoteType {
    let mut rng = rand::thread_rng();
    let vote_type: VoteType = if rng.gen() {
        VoteType::Yes
    } else {
        VoteType::No
    };
    vote_type
}
