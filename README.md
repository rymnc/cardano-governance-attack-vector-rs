# cardano-governance-attack-vector-rs
Naive implementation of cardano's governance attack vector

## Steps:

Pre Voting Phase

1. Voters, and their keys are created
2. The voting round is created
3. Voters register to the voting round
4. Committee members are randomly selected

Voting Phase

5. Voters submit their votes

Post-Voting Phase

6. Voters votes are tallied by the committee members
7. Committee members start generating part of their keys
8. Committee members finish generation



9. Committee members "confirm the key" - revealing their portion of the key
10. Threshold for revealing the key has not been met - which stalls the next round from being created
11. One more member confirms the key
12. The secret can be generated now

## Usage

1. `cargo run`
