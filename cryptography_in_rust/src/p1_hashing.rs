#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use hex::ToHex;
// You can find the hashing algorithms in the exports from sp_core. In order to easily see what is
// available from sp_core, it might be helpful to look at the rust docs:
// https://paritytech.github.io/substrate/master/sp_core/index.html
use sp_core::*;

/// For simplicity in this exercise, we are only working with 128-bit hashes.
const HASH_SIZE: usize = 16;

/// Use the blake2 hashing algorithm to calculate the 128-bit hash of some input data
pub fn hash_with_blake(data: &[u8]) -> [u8; HASH_SIZE] {
    hashing::blake2_128(data)
}

/// Use the twox hashing algorithm to calculate the 128-bit hash of some input data
pub fn hash_with_twox(data: &[u8]) -> [u8; HASH_SIZE] {
    hashing::twox_128(data)
}

#[derive(Clone, PartialEq, Eq)]
pub enum HashAlgo {
    TwoX,
    Blake2,
}

/// Use the hashing algorithm variant specified in the argument to hash the data
pub fn hash_with(data: &[u8], algorithm: HashAlgo) -> [u8; HASH_SIZE] {
    match algorithm {
        HashAlgo::TwoX => hash_with_twox(data),
        HashAlgo::Blake2 => hash_with_blake(data),
    }
}

/// Return true iff data is the preimage of hash under the specified algorithm
pub fn is_hash_preimage(hash: [u8; HASH_SIZE], data: &[u8], algorithm: HashAlgo) -> bool {
    hash.eq(&hash_with(data, algorithm))
}

/// Add an integrity check to some data by using the blake2 hashing algorithm.
///
/// Hashes can also be used to check data integrity! We will implement a version of this using the
/// blake2 hashing algorithm. To append an integrity code to the end of some input, hash the data,
/// and append the 128-bit hash to the data. The result will look like `data | hash(data)`, using |
/// for concatenation.
pub fn add_integrity_check(data: &[u8]) -> Vec<u8> {
    [data, &hash_with_blake(data)].concat()
}

/// Verify the integrity of some data via the checksum, and return the original data
///
/// In order to verify that the data is valid, we separate it out into the received hash and the
/// original data. Then, we hash the original data and compare it to the received hash. If it is
/// the same, we return the original data. Otherwise, we return an error.
///
/// Note that when receiving data that has an integrity check, it is important that we know
/// _exactly_ how the integrity check was generated. Most of the time, the integrity checks are
/// not able to be self-describing, so the verification end needs to know how to use the
/// integrity check.
pub fn verify_data_integrity(data: Vec<u8>) -> Result<Vec<u8>, ()> {
    let data_slice = data.as_slice();
    let len = data_slice.len();
    if len <= 16 || data_slice[len - 16..len] != hash_with_blake(&data_slice[..len - 16]) {
        Err(())
    } else {
        Ok(data_slice[..len - 16].to_vec())
    }
}

use rand::{rngs::SmallRng, seq::IteratorRandom, Rng, SeedableRng};
use std::{cell::RefCell, collections::HashMap};
use strum::{EnumIter, IntoEnumIterator};
type HashValue = [u8; HASH_SIZE];

/// Now that we are comfortable using hashes, let's implement a classic commit-reveal scheme using a
/// public message board. This message board implements some functionality to allow people to communicate.
/// It allows people to commit to a message, and then later reveal that message. It also lets people
/// look up a commitment to see if the message has been revealed or not.
///
/// This message board will use the 128-bit Blake2 hashing algorithm.
#[derive(Debug)]
pub struct PublicMessageBoard {
    /// The commitals to this public message board. A 'None' value represents a commitment that has
    /// not been revealed. A 'Some' value will contain the revealed value corresponding to the
    /// commitment.
    commitals: HashMap<HashValue, Option<String>>,
    /// A seeded RNG used to generate randomness for committing
    ///
    /// STUDENTS: DO NOT USE THIS YOURSELF. The provided code already uses it everywhere necessary.
    rng: SmallRng,
}

impl PublicMessageBoard {
    /// Create a new message board
    pub fn new(rng_seed: u64) -> Self {
        PublicMessageBoard {
            commitals: HashMap::new(),
            rng: SmallRng::seed_from_u64(rng_seed),
        }
    }

    /// Post a commitment to the public message board, returning the message with added randomness
    /// and the commitment to share. If the commitment already exists, this does not modify the
    /// board, but returns the same values.
    ///
    /// The input messages should have some randomness appended to them so that an attacker cannot
    /// guess the messages to crack the hash. For compatibility with tests, do not use the message
    /// board's RNG other than the provided code below.
    ///
    /// Note that in reality, the commitment would be calculated offline, and only the commitment
    /// posted to the message board. However, in this example, we pretend that this is a "frontend"
    /// to the message board that handles that for you.
    pub fn post_commitment(&mut self, msg: String) -> (String, HashValue) {
        let randomness: [u8; 4] = self.rng.gen();
        let randomness_string = hex::encode(randomness);
        let commitment = Self::reveal_to_commit(&format!("{}{}", msg, randomness_string));
        let message = format!("{}{}", msg, randomness_string);
        if !self.commitals.contains_key(&commitment) {
            self.commitals.insert(commitment, None);
        }
        (message, commitment)
    }

    /// Post a reveal for an existing commitment. The input should be the message with randomness added.
    ///
    /// Returns Ok(commitment) if the reveal was successful, or an error if the commitment wasn't
    /// found or has already been revealed.
    pub fn post_reveal(&mut self, committed_msg: String) -> Result<HashValue, ()> {
        let commitment = Self::reveal_to_commit(&committed_msg);
        match self.commitals.get_mut(&commitment) {
            Some(val) => match val {
                Some(_) => Err(()),
                None => {
                    *val = Some(committed_msg);
                    Ok(commitment)
                }
            },
            None => Err(()),
        }
    }

    /// Check a certain commitment. Errors if the commitment doesn't exist, and otherwise returns
    /// None if the commitment has not been revealed, or the value if it has been revealed.
    pub fn check_commitment(&self, commitment: HashValue) -> Result<Option<String>, ()> {
        match self.commitals.get(&commitment) {
            Some(val) => Ok(val.clone()),
            None => Err(()),
        }
    }

    /// Helper method to convert from a reveal to the corresopnding commitment.
    pub fn reveal_to_commit(reveal: &str) -> HashValue {
        hash_with_blake(reveal.as_bytes())
    }
}

/// Now, we will use our message board to play a game of rock paper scissors!
///
/// This enum tracks the game state. The game will always go in the following order:
///
/// 1. Player 1 commits to their play.
/// 2. Player 2 commits to their play.
/// 3. Player 1 reveals their play.
/// 4. Player 2 reveals their play. At this point, the game is over and either player 1 or player 2
///     has won!
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RPSGameState {
    NotStarted,
    Player1Committed(HashValue),
    BothCommitted {
        p1_commit: HashValue,
        p2_commit: HashValue,
    },
    Player1Revealed {
        p1_reveal: String,
        p2_commit: HashValue,
    },
    Completed {
        p1_reveal: String,
        p2_reveal: String,
    },
}

impl RPSGameState {
    // Return the winner if there is one, or none if it is a tie. Errors if the game state is not
    // terminal, or the committed strings are malformed.
    pub fn winner(state: RPSGameState) -> Result<Option<PlayerNumber>, ()> {
        match state {
            Self::Completed {
                p1_reveal,
                p2_reveal,
            } => {
                // If one of the players made an invalid play, we automatically say the other player won.
                // We first do that logic for player 1, because they must have broken the rules of
                // the game first.
                let p1_play = match RPSPlay::from_string_with_randomness(&p1_reveal) {
                    Ok(play) => play,
                    Err(_) => {
                        return Ok(Some(PlayerNumber::Second));
                    }
                };
                let p2_play = match RPSPlay::from_string_with_randomness(&p2_reveal) {
                    Ok(play) => play,
                    Err(_) => {
                        return Ok(Some(PlayerNumber::First));
                    }
                };
                match (p1_play, p2_play) {
                    (RPSPlay::Rock, RPSPlay::Scissors) => Ok(Some(PlayerNumber::First)),
                    (RPSPlay::Rock, RPSPlay::Paper) => Ok(Some(PlayerNumber::Second)),
                    (RPSPlay::Paper, RPSPlay::Rock) => Ok(Some(PlayerNumber::First)),
                    (RPSPlay::Paper, RPSPlay::Scissors) => Ok(Some(PlayerNumber::Second)),
                    (RPSPlay::Scissors, RPSPlay::Paper) => Ok(Some(PlayerNumber::First)),
                    (RPSPlay::Scissors, RPSPlay::Rock) => Ok(Some(PlayerNumber::Second)),
                    _ => Ok(None),
                }
            }
            _ => Err(()),
        }
    }
}

/// The possible player numbers in rock paper scissors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayerNumber {
    First,
    Second,
}

/// The possible plays in a game of rock paper scissors
#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum RPSPlay {
    Rock,
    Paper,
    Scissors,
}

impl std::fmt::Display for RPSPlay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl RPSPlay {
    /// Convert a string with 4 bytes of hex-encoded randomness at the end into an RPS play
    pub fn from_string_with_randomness(s: &str) -> Result<Self, ()> {
        if s.len() - 4 * 2 < "Rock".len() || s.len() - 4 * 2 > "Scissors".len() {
            return Err(());
        }
        match &s[..s.len() - 8] {
            "Rock" => Ok(RPSPlay::Rock),
            "Paper" => Ok(RPSPlay::Paper),
            "Scissors" => Ok(RPSPlay::Scissors),
            _ => return Err(()),
        }
    }
}

/// A careful player of rock paper scissors, who only plays if the game state is correct.
pub struct RPSPlayer<'a> {
    /// The message board that this rock paper scissors player uses to communicate with the other
    /// player.
    ///
    /// This can be used mutably by using `self.message_board.borrow_mut()`.
    message_board: &'a RefCell<PublicMessageBoard>,
    /// If this player is playing first or second
    player_number: PlayerNumber,
    /// The string used to commit, with included randomness. This will always be the string
    /// representation of an RPSPlay
    previous_commitment_str: Option<String>,
    /// The commitment made by the other player, once seen. It is important for the player to keep
    /// this information, because it is not safely tracked in the game state.
    other_commit: Option<HashValue>,
    /// A seeded RNG used to generate randomness for deciding on a play.
    ///
    /// STUDENTS: DO NOT USE THIS YOURSELF. The provided code already uses it everywhere necessary.
    rng: SmallRng,
}

impl<'a> RPSPlayer<'a> {
    /// Create a new player to use in a RPS game.
    pub fn new(
        rng_seed: u64,
        message_board: &'a RefCell<PublicMessageBoard>,
        player_order: PlayerNumber,
    ) -> Self {
        RPSPlayer {
            message_board,
            player_number: player_order,
            previous_commitment_str: None,
            other_commit: None,
            rng: SmallRng::seed_from_u64(rng_seed),
        }
    }

    /// Make the next play as a careful player in a rock paper scissors game. You should only return
    /// the new game state if the old game state is consistent with the state of the message board
    /// and of the internal player state, otherwise error.
    ///
    /// In particular, make note of the following things:
    /// - If player 2 is committing, make sure that player 1 has already committed to
    ///     the message board. If not, error.
    /// - If player 1 is revealing, make sure that player 2 has already committed to
    ///     the message board. If not, error.
    /// - If player 2 is revealing, make sure that player 1 has already revealed to
    ///     the message board.
    /// - Once a player has seen the other player's commitment, make sure it is consistent
    ///     with any future game states. If it ever fails to be consistent, error.
    /// - DO NOT USE THE RANDOMNESS YOURSELF. This _will_ break automated tests.
    pub fn progress_game(&mut self, state: RPSGameState) -> Result<RPSGameState, ()> {
        // The student starter code is each match arm up to the `todo!()`.
        match state {
            RPSGameState::NotStarted if self.player_number == PlayerNumber::First => {
                let play = RPSPlay::iter().choose_stable(&mut self.rng).unwrap();
                let (message, commitment) = self
                    .message_board
                    .borrow_mut()
                    .post_commitment(play.to_string());
                self.previous_commitment_str = Some(message);
                Ok(RPSGameState::Player1Committed(commitment))
            }
            RPSGameState::Player1Committed(p1_commit)
                if self.player_number == PlayerNumber::Second =>
            {
                let play = RPSPlay::iter().choose_stable(&mut self.rng).unwrap();
                let (message, commitment) = self
                    .message_board
                    .borrow_mut()
                    .post_commitment(play.to_string());
                self.previous_commitment_str = Some(message);
                self.other_commit = Some(p1_commit);
                Ok(RPSGameState::BothCommitted {
                    p1_commit,
                    p2_commit: commitment,
                })
            }
            RPSGameState::BothCommitted {
                p1_commit,
                p2_commit,
            } if self.player_number == PlayerNumber::First => {
                if let Some(commitment_str) = &self.previous_commitment_str {
                    let _ = self
                        .message_board
                        .borrow_mut()
                        .post_reveal(commitment_str.to_string())?;
                    Ok(RPSGameState::Player1Revealed {
                        p1_reveal: commitment_str.clone(),
                        p2_commit,
                    })
                } else {
                    Err(())
                }
            }
            RPSGameState::Player1Revealed {
                p1_reveal: p1,
                p2_commit: p2,
            } if self.player_number == PlayerNumber::Second => {
                if let Some(commitment_str) = &self.previous_commitment_str {
                    let _ = self
                        .message_board
                        .borrow_mut()
                        .post_reveal(commitment_str.to_string())?;
                    Ok(RPSGameState::Completed {
                        p1_reveal: p1,
                        p2_reveal: commitment_str.clone(),
                    })
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

/// This function is not graded. It is just for collecting feedback.
/// On a scale from 0 - 100, with zero being extremely easy and 100 being extremely hard, how hard
/// did you find the exercises in this section?
pub fn how_hard_was_this_section() -> u8 {
    70
}

/// This function is not graded. It is just for collecting feedback.
/// About how much time (in hours) did you spend on the exercises in this section?
pub fn how_many_hours_did_you_spend_on_this_section() -> f32 {
    4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_integrity_no_panics() {
        // This test might panic if they didn't check bounds before slicing
        let too_short_data = b"less than 16";
        assert!(verify_data_integrity(too_short_data.to_vec()).is_err())
    }

    #[test]
    fn hash_with_blake2_test() {
        let data = b"PBA Berkeley 2023!";
        let hash = hash_with_blake(data);
        let expected = hex::decode("47ab80e805a80033b7e0587ceb5c575d").unwrap();
        assert_eq!(&expected, &hash);
    }

    #[test]
    fn hash_with_twox_test() {
        let data = b"PBA Berkeley 2023!";
        let hash = hash_with_twox(data);
        let expected = hex::decode("c8a3248a3f671d43c01251a28494903c").unwrap();
        assert_eq!(&expected, &hash);
    }

    #[test]
    fn hash_with_test() {
        let data = b"PBA Berkeley 2023!";

        let twox_hash = hash_with(data, HashAlgo::TwoX);
        let expected_twox = hex::decode("c8a3248a3f671d43c01251a28494903c").unwrap();
        assert_eq!(&expected_twox, &twox_hash);

        let blake_hash = hash_with(data, HashAlgo::Blake2);
        let expected_blake = hex::decode("47ab80e805a80033b7e0587ceb5c575d").unwrap();
        assert_eq!(&expected_blake, &blake_hash);
    }

    #[test]
    fn hash_preimage_test() {
        let data = b"PBA Berkeley 2023!";
        // This data has been altered, misspelling "Berkeley"
        let bad_data = b"PBA Berkley 2023!";

        let mut twox_hash = [0u8; HASH_SIZE];
        hex::decode_to_slice("c8a3248a3f671d43c01251a28494903c", &mut twox_hash).unwrap();
        let mut blake_hash = [0u8; HASH_SIZE];
        hex::decode_to_slice("47ab80e805a80033b7e0587ceb5c575d", &mut blake_hash).unwrap();

        // works on actual data
        assert!(is_hash_preimage(twox_hash.clone(), data, HashAlgo::TwoX));
        assert!(is_hash_preimage(blake_hash.clone(), data, HashAlgo::Blake2));

        // Must be correct hashing algorithm
        assert!(!is_hash_preimage(blake_hash.clone(), data, HashAlgo::TwoX));

        // altered data doesn't verify, even though it's only 1 character off
        assert!(!is_hash_preimage(twox_hash, bad_data, HashAlgo::TwoX));
        assert!(!is_hash_preimage(blake_hash, bad_data, HashAlgo::Blake2));
    }

    #[test]
    fn add_integrity_check_test() {
        let data = b"PBA Berkeley 2023!";
        let blake = hex::decode("47ab80e805a80033b7e0587ceb5c575d").unwrap();
        let mut expected = Vec::new();
        expected.extend(data);
        expected.extend(blake);

        let integrity_checked_data = add_integrity_check(data);
        assert_eq!(data.len() + HASH_SIZE, integrity_checked_data.len());
        assert_eq!(expected, integrity_checked_data);
    }

    #[test]
    fn verify_integrity_test() {
        let data = b"PBA Berkeley 2023!";
        let blake = hex::decode("47ab80e805a80033b7e0587ceb5c575d").unwrap();
        let mut integrity_checked_data = Vec::new();
        integrity_checked_data.extend(data);
        integrity_checked_data.extend(blake);

        let mut bad_data = integrity_checked_data.clone();
        bad_data.remove(15);

        let integrity_checked_result = verify_data_integrity(integrity_checked_data).unwrap();
        assert_eq!(&data[..], &integrity_checked_result);
        assert!(verify_data_integrity(bad_data).is_err());
    }
}

#[cfg(test)]
mod optional_tests {
    use super::*;

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn reveal_to_commit_test() {
        let input = "PBA Berkeley 2023!";
        let expected = hex::decode("47ab80e805a80033b7e0587ceb5c575d").unwrap();
        assert_eq!(&expected, &PublicMessageBoard::reveal_to_commit(input));
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn post_commitment_test() {
        let rng_seed = 2023;
        let mut pmb = PublicMessageBoard::new(rng_seed);

        let mut test_rng = SmallRng::seed_from_u64(rng_seed);
        let randomness: [u8; 4] = test_rng.gen();
        let randomness_string = hex::encode(randomness);

        let post = "PBA Berkeley 2023!".to_string();
        let (message, commit) = pmb.post_commitment(post.clone());
        assert_eq!(format!("{}{}", post.clone(), randomness_string), message);
        assert_eq!(blake2_128(message.as_bytes()), commit);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn post_reveal_test() {
        let rng_seed = 2023;
        let mut pmb = PublicMessageBoard::new(rng_seed);

        let post = "PBA Berkeley 2023!".to_string();
        let (message, commit) = pmb.post_commitment(post.clone());
        let commit2 = pmb.post_reveal(message.clone());
        assert_eq!(Ok(commit), commit2);

        // this has not been committed first
        let bad_post = "PBA Cambridge 2022!".to_string();
        assert!(pmb.post_reveal(bad_post).is_err());
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn check_commitment_test() {
        let rng_seed = 2023;
        let mut pmb = PublicMessageBoard::new(rng_seed);

        let mut test_rng = SmallRng::seed_from_u64(rng_seed);
        let randomness: [u8; 4] = test_rng.gen();
        let randomness_string = hex::encode(randomness);

        let post = "PBA Berkeley 2023!".to_string();
        let (message, commit) = pmb.post_commitment(post.clone());

        assert_eq!(pmb.check_commitment(commit.clone()), Ok(None));
        assert_eq!(pmb.check_commitment([5u8; 16]), Err(()));

        pmb.post_reveal(message.clone()).unwrap();
        assert_eq!(pmb.check_commitment(commit), Ok(Some(message)));
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_play_decode_strings() {
        let rock = "Rock00000000";
        let paper = "Paper00000000";
        let scissors = "Scissors00000000";
        assert_eq!(
            Ok(RPSPlay::Rock),
            RPSPlay::from_string_with_randomness(rock)
        );
        assert_eq!(
            Ok(RPSPlay::Paper),
            RPSPlay::from_string_with_randomness(paper)
        );
        assert_eq!(
            Ok(RPSPlay::Scissors),
            RPSPlay::from_string_with_randomness(scissors)
        );
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_play_decode_rejects_properly() {
        let not_at_start = "0Rock0Paper0";
        let wrong_randomness_length = "Paper000";
        assert!(RPSPlay::from_string_with_randomness(not_at_start).is_err());
        assert!(RPSPlay::from_string_with_randomness(wrong_randomness_length).is_err());

        let pmb = PublicMessageBoard::new(5);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_game_test_1() {
        let rng_seed = 2023;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);

        // Because SmallRng is not necessarily deterministic across platforms, we need to replicate
        // the RNG calls in the RPS player and create an identically seeded message board in order
        // to know what play to expect in a test.
        let mut pmb2 = PublicMessageBoard::new(rng_seed);
        let mut p1_test_rng = SmallRng::seed_from_u64(rng_seed);
        let p1_expected_play = RPSPlay::iter().choose_stable(&mut p1_test_rng).unwrap();
        let (_, p1_commit) = pmb2.post_commitment(p1_expected_play.to_string());
        let expected = RPSGameState::Player1Committed(p1_commit);

        let mut p1 = RPSPlayer::new(rng_seed, &pmb_refcell, PlayerNumber::First);
        let state2 = p1.progress_game(RPSGameState::NotStarted).unwrap();
        assert_eq!(expected, state2);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_player_test_2() {
        let rng_seed = 2023;
        let p2_rng_seed = 2024;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);

        // Because SmallRng is not necessarily deterministic across platforms, we need to replicate
        // the RNG calls in the RPS player and create an identically seeded message board in order
        // to know what play to expect in a test.
        let mut pmb2 = PublicMessageBoard::new(rng_seed);
        let mut p1_test_rng = SmallRng::seed_from_u64(rng_seed);
        let mut p2_test_rng = SmallRng::seed_from_u64(p2_rng_seed);
        let p1_expected_play = RPSPlay::iter().choose_stable(&mut p1_test_rng).unwrap();
        let p2_expected_play = RPSPlay::iter().choose_stable(&mut p2_test_rng).unwrap();

        let (_, p1_commit) = pmb2.post_commitment(p1_expected_play.to_string());
        let (_, p2_commit) = pmb2.post_commitment(p2_expected_play.to_string());

        let expected = RPSGameState::BothCommitted {
            p1_commit,
            p2_commit,
        };

        let mut p1 = RPSPlayer::new(rng_seed, &pmb_refcell, PlayerNumber::First);
        let mut p2 = RPSPlayer::new(p2_rng_seed, &pmb_refcell, PlayerNumber::Second);
        let state2 = p1.progress_game(RPSGameState::NotStarted).unwrap();
        let state3 = p2.progress_game(state2).unwrap();
        assert_eq!(expected, state3);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_player_test_3() {
        let rng_seed = 2023;
        let p2_rng_seed = 2024;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);

        // Because SmallRng is not necessarily deterministic across platforms, we need to replicate
        // the RNG calls in the RPS player and create an identically seeded message board in order
        // to know what play to expect in a test.
        let mut pmb2 = PublicMessageBoard::new(rng_seed);
        let mut p1_test_rng = SmallRng::seed_from_u64(rng_seed);
        let mut p2_test_rng = SmallRng::seed_from_u64(p2_rng_seed);
        let p1_expected_play = RPSPlay::iter().choose_stable(&mut p1_test_rng).unwrap();
        let p2_expected_play = RPSPlay::iter().choose_stable(&mut p2_test_rng).unwrap();

        let (p1_reveal, _) = pmb2.post_commitment(p1_expected_play.to_string());
        let (_, p2_commit) = pmb2.post_commitment(p2_expected_play.to_string());

        let expected = RPSGameState::Player1Revealed {
            p1_reveal,
            p2_commit,
        };

        let mut p1 = RPSPlayer::new(rng_seed, &pmb_refcell, PlayerNumber::First);
        let mut p2 = RPSPlayer::new(p2_rng_seed, &pmb_refcell, PlayerNumber::Second);
        let state2 = p1.progress_game(RPSGameState::NotStarted).unwrap();
        let state3 = p2.progress_game(state2).unwrap();
        let state4 = p1.progress_game(state3).unwrap();
        assert_eq!(expected, state4);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_player_full_game_test() {
        let rng_seed = 2023;
        let p2_rng_seed = 2024;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);

        // Because SmallRng is not necessarily deterministic across platforms, we need to replicate
        // the RNG calls in the RPS player and create an identically seeded message board in order
        // to know what play to expect in a test.
        let mut pmb2 = PublicMessageBoard::new(rng_seed);
        let mut p1_test_rng = SmallRng::seed_from_u64(rng_seed);
        let mut p2_test_rng = SmallRng::seed_from_u64(p2_rng_seed);
        let p1_expected_play = RPSPlay::iter().choose_stable(&mut p1_test_rng).unwrap();
        let p2_expected_play = RPSPlay::iter().choose_stable(&mut p2_test_rng).unwrap();

        let (p1_reveal, _) = pmb2.post_commitment(p1_expected_play.to_string());
        let (p2_reveal, _) = pmb2.post_commitment(p2_expected_play.to_string());

        let expected = RPSGameState::Completed {
            p1_reveal,
            p2_reveal,
        };

        let mut p1 = RPSPlayer::new(rng_seed, &pmb_refcell, PlayerNumber::First);
        let mut p2 = RPSPlayer::new(p2_rng_seed, &pmb_refcell, PlayerNumber::Second);
        let state2 = p1.progress_game(RPSGameState::NotStarted).unwrap();
        let state3 = p2.progress_game(state2).unwrap();
        let state4 = p1.progress_game(state3).unwrap();
        let state5 = p2.progress_game(state4).unwrap();
        assert_eq!(expected, state5);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_initial_failures_test() {
        let rng_seed = 2023;
        let p2_rng_seed = 2024;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);
        let mut p2 = RPSPlayer::new(p2_rng_seed, &pmb_refcell, PlayerNumber::Second);

        // This test only covers some of the possible ways this could fail, so make sure to test on
        // your own!

        // p1's listed commit isn't on the message board!
        assert!(p2
            .progress_game(RPSGameState::Player1Committed([5u8; HASH_SIZE]))
            .is_err());
        // p1 has to start the game
        assert!(p2.progress_game(RPSGameState::NotStarted).is_err());
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn rps_progress_mismatch_failures_test() {
        let rng_seed = 2023;
        let p2_rng_seed = 2024;
        let pmb = PublicMessageBoard::new(rng_seed);
        let pmb_refcell = RefCell::new(pmb);
        let mut p1 = RPSPlayer::new(rng_seed, &pmb_refcell, PlayerNumber::First);
        let mut p2 = RPSPlayer::new(p2_rng_seed, &pmb_refcell, PlayerNumber::Second);

        // This test only covers some of the possible ways this could fail, so make sure to test on
        // your own!

        let state2 = p1.progress_game(RPSGameState::NotStarted).unwrap();
        let state3 = p2.progress_game(state2).unwrap();
        let (p1_commit, p2_commit) = match state3.clone() {
            RPSGameState::BothCommitted {
                p1_commit,
                p2_commit,
            } => (p1_commit, p2_commit),
            _ => panic!("state3 should be both committed"),
        };

        // P1's previous commit doesn't match p1's current commit
        let bad_state3 = RPSGameState::BothCommitted {
            p1_commit: [5u8; HASH_SIZE],
            p2_commit: p2_commit.clone(),
        };
        assert!(p1.progress_game(bad_state3).is_err());

        // P1's reveal doesn't match up with their previous commit
        let bad_state4 = RPSGameState::Player1Revealed {
            p1_reveal: "Paper12121212".to_string(),
            p2_commit,
        };
        assert!(p2.progress_game(bad_state4).is_err());
    }
}
