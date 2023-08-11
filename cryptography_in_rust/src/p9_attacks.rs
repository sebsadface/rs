#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod low_entropy_hash {
    //! This section will focus on taking advantage of low entropy in a hash-based commit-reveal
    //! scheme.
    //!
    //! Suppose you are playing a game with your friend Bob over the internet. Bob will pick a
    //! random number between 1 and 100, and you will guess it. However, you need a way to make sure
    //! that you're both honest! Bob suggests a classic hash-based commit-reveal scheme. He will
    //! pick his number, and send you the blake2_256 hash of it. Then, you will guess a number, and
    //! he can reveal the original number. Because you can check the hash, you will know that he
    //! didn't cheat!
    //!
    //! However, you have a special way to always win this game.
    use sp_core::blake2_256;

    /// This is the code that Bob will use to commit to a guess. He thinks that, because it is a
    /// 32-byte hash, it is impossible to brute force.
    pub fn bob_commits_to_guess(n: u8) -> [u8; 32] {
        let guess_vec = vec![n];
        blake2_256(&guess_vec)
    }

    /// Based on Bob's commit, calculate a guess. You should never lose!
    pub fn guess_based_on_hash(hash: [u8; 32]) -> u8 {
        for i in 1..=100 {
            if blake2_256(&vec![i]) == hash {
                return i;
            }
        }
        panic!("This should be impossible if bob guessed a number between 1 and 100")
    }

    /// Bob realized that the problem was probably that the input to the hash was too short!  So he
    /// decided to append the hash of the number to the number itself, and then hash that.  This
    /// way, the input to the hash is 33 bytes! There's no way you'll be able to crack that, right?
    pub fn bob_commits_to_guess_v2(n: u8) -> [u8; 32] {
        let mut guess_vec = vec![n];
        let hash_of_guess = blake2_256(&guess_vec);
        guess_vec.extend(&hash_of_guess);
        blake2_256(&guess_vec)
    }

    /// Based on Bob's committal using his second scheme, calculate a guess. You should
    /// still be able to never lose!
    pub fn guess_based_on_hash_v2(hash: [u8; 32]) -> u8 {
        for i in 1..=100 {
            let mut guess_vec = vec![i];
            let hash_of_guess = blake2_256(&guess_vec);
            guess_vec.extend(&hash_of_guess);
            if blake2_256(&guess_vec) == hash {
                return i;
            }
        }
        panic!("This should be impossible if bob guessed a number between 1 and 100")
    }

    #[test]
    fn always_guess_correctly() {
        let h1 = bob_commits_to_guess(1);
        assert_eq!(1, guess_based_on_hash(h1));

        let h17 = bob_commits_to_guess(17);
        assert_eq!(17, guess_based_on_hash(h17));

        let h80 = bob_commits_to_guess(80);
        assert_eq!(80, guess_based_on_hash(h80));
    }

    #[test]
    fn still_always_guess_correctly() {
        let h1 = bob_commits_to_guess_v2(1);
        assert_eq!(1, guess_based_on_hash_v2(h1));

        let h17 = bob_commits_to_guess_v2(17);
        assert_eq!(17, guess_based_on_hash_v2(h17));

        let h80 = bob_commits_to_guess_v2(80);
        assert_eq!(80, guess_based_on_hash_v2(h80));
    }
}

pub mod bad_signing_format {
    //! This section focuses on abusing bad formats for signed messages.

    use sp_core::sr25519::{Pair, Public, Signature};
    use sp_core::Pair as PairT;
    use sp_runtime::traits::Verify;

    type AccountId = u64;

    #[derive(Clone, Debug, PartialEq, Eq)]
    /// A message format for a transaction
    pub struct TransactionMessage {
        /// The amount of currency to send
        pub amount: u64,
        /// The recipient of the transaction, represented as an account identifier
        pub recipient: AccountId,
    }

    impl TransactionMessage {
        /// Serialize this transaction into bytes
        pub fn serialize(&self) -> Vec<u8> {
            format!("{}{}", self.amount, self.recipient).into_bytes()
        }
    }

    /// Returns true iff the transaction is valid, and represents the intention of the signer.
    ///
    /// DO NOT CHANGE THIS METHOD
    pub fn validate_transaction(
        message: &TransactionMessage,
        sig: Signature,
        sender: Public,
    ) -> bool {
        let msg_bytes = message.serialize();
        sig.verify(msg_bytes.as_slice(), &sender)
    }

    /// Suppose you know that alice is going to send a transaction for 10 to account number 54321.
    ///
    /// Create a second message which will also be accepted by the system, which does something
    /// _different_ to Alice's account.
    pub fn create_second_message() -> TransactionMessage {
        TransactionMessage {
            amount: 10543,
            recipient: 21,
        }
    }

    /// Now, suppose there is some silly game that you convinced Alice to play. Alice doesn't trust
    /// the `TransactionMessage` anymore, and so she won't sign any messages for that. However, she
    /// does still have a lot of money in her account.
    ///
    /// In this game, you sign messages with your account to guess the number of red, green, and
    /// blue beans inside of a virtual glass jar. Then, you have the option to donate any winnings
    /// (if you win) to charity.
    pub struct SillyGameMessage {
        /// The number of red beans in the jar
        red_beans: u8,
        /// The number of blue beans in the jar
        blue_beans: u8,
        /// The number of green beans in the jar
        green_beans: u8,
        /// What percent of the money, out of 100, to donate to charity
        percent_to_charity: u8,
    }

    impl SillyGameMessage {
        /// This message takes serialization more seriously. There is no way to make these messages
        /// overlap, because each field takes up exactly 1 byte of space!
        pub fn serialize(&self) -> Vec<u8> {
            vec![
                self.red_beans,
                self.blue_beans,
                self.green_beans,
                self.percent_to_charity,
            ]
        }
    }

    /// Alice is willing to sign any message for the silly game, because she does not think it is
    /// important.
    pub fn get_alice_signature(message: &SillyGameMessage) -> (Signature, Public) {
        let alice = <Pair as PairT>::from_string("//Alice", None).unwrap();
        if message.percent_to_charity > 100 {
            panic!("That message doesn't make sense!")
        }
        let msg_bytes = message.serialize();
        (alice.sign(&msg_bytes), alice.public())
    }

    /// Get a signature from alice that can be used for a transaction message to send 10 dollars to
    /// your account 21. You are not allowed to use any functions dealing with signatures other than
    /// `get_alice_signature`.
    pub fn get_money_from_silly_game() -> (TransactionMessage, Signature, Public) {
        // Hint: In rust, strings are by default utf-8 encoded
        let (signiture, public) = get_alice_signature(&SillyGameMessage {
            red_beans: 49,          // ascii '1'
            blue_beans: 48,         // ascii '0'
            green_beans: 50,        // ascii '2'
            percent_to_charity: 49, // ascii '1'
        });

        let transaction_message = TransactionMessage {
            amount: 10,
            recipient: 21,
        };

        (transaction_message, signiture, public)
    }

    #[test]
    fn messages_both_validate() {
        let alice_msg = TransactionMessage {
            amount: 10,
            recipient: 54321,
        };
        let alice_msg_bytes = alice_msg.serialize();
        let alice = <Pair as PairT>::from_string("//Alice", None).unwrap();

        let alice_sig = alice.sign(&alice_msg_bytes);

        assert!(validate_transaction(
            &alice_msg,
            alice_sig.clone(),
            alice.public()
        ));

        let other_msg = create_second_message();
        assert!(validate_transaction(&other_msg, alice_sig, alice.public()));
        assert_ne!(alice_msg, other_msg);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn silly_message_abuse() {
        let alice = <Pair as PairT>::from_string("//Alice", None).unwrap();

        let (msg, sig, public) = get_money_from_silly_game();
        assert_eq!(
            TransactionMessage {
                amount: 10,
                recipient: 21
            },
            msg.clone()
        );
        assert_eq!(alice.public(), public.clone());
        assert!(validate_transaction(&msg, sig, public));
    }
}

pub mod timing_attacks {
    //! This section will focus on implementing a timing attack against the insecure password
    //! checker provided. The timing attack here will be on the length of time it takes the checker
    //! to return if a guess is correct. It will be slower based on how many characters in the guess
    //! are correct.
    //!
    //! Some tips for implementing and running a timing attack:
    //! - You will need to do many trials of each operation in order to get a large enough sample size to
    //!     have a measurable time difference. The time can be impacted by OS scheduling, cache effects,
    //!     imprecision in measurement, and other issues. For that reason, I had successes only when running
    //!     ~1_000_000 trials if using a debug build, and ~10_000_000 trials when using a release build.
    //! - You can measure times in rust by using `let now = Instant::now()` before starting the block of
    //!     code to measured, and `now.elapsed().as_nanos()` after the block. This will give the number of
    //!     nanoseconds elapsed as a u128.
    //! - You can make your tests run much, much faster even with more trials by running it in release mode.
    //!     Use the command `cargo test --release --package pba-assignment --features run-slow-tests --lib -- p9_attacks::crack_password_test --exact --nocapture`
    //!     However, if you do this, you need to make sure that the compiler doesn't compile out the inner
    //!     loop that is timed.  This can be circumvented by making it so it cannot be ignored, by using the
    //!     below as your inner loop code.
    //!     ```ignore
    //!     let corr = checker.is_correct(&guess);
    //!     if corr {
    //!         return {whatever you return from your function};
    //!     }
    //!     ```
    //! - Including print statements that show the progress of the cracking can be helpful if you're not sure why it failed.
    //! - Consider if the code will still take a different amount of time for the last character! If not,
    //!     you might need to take a different approach for the last character.
    //! - If you are having issues where a specific character takes an unexpectedly long time when testing,
    //!     and ruins your password cracking, you can try having a "runoff" to correct for one-off effects like
    //!     that. In order to take this approach, track the best 2 candidates when iterating over possible
    //!     characters, and test them again at the end, choosing the better of the two. This will not fix
    //!     _every_ failure, but it should fix most.
    //! In total, the staff solution running on an M1 pro chip can crack a 15-character password in about 25
    //! seconds, using 10 million trials per character guessed. It can crack a 7 character password in 6
    //! seconds.
    //!
    use std::time::Instant;

    /// The maximum password length.
    const MAX_PASSWORD_LEN: usize = 20;

    /// An insecure implementation of a password checker. This checker holds an internal password, and
    /// compares guesses to it.
    pub struct InsecurePasswordChecker {
        password: Vec<u8>,
    }

    impl InsecurePasswordChecker {
        /// Return if a password is correct
        ///
        /// This is a little extra-bad coding with respect to a timing attack, because it makes it much
        /// faster to carry out a timing attack. Although it is absolutely still possible to perform a
        /// timing attack, it takes a lot longer to run enough samples to see an appreciable difference.
        /// Additionally, the compiler is very liable to optimize things to the point it becomes
        /// difficult to distinguish with things like SIMD instructions, depending on the architecture.
        pub fn is_correct(&self, guess: &[u8]) -> bool {
            if self.password.len() != guess.len() {
                false
            } else {
                for i in 0..guess.len() {
                    if self.password.get(i) != guess.get(i) {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// An iterator over all u8s that are possible in an alphanumeric password.
    pub fn alphanumeric_iterator() -> impl Iterator<Item = u8> {
        // (lowercase latin letters).chain(uppercase latin letters).chain(digits)
        (97u8..=122u8).chain(65u8..=90).chain(48u8..=57u8)
    }

    /*
    The following two functions will focus on implementing a timing attack against the insecure password
    checker above. The timing attack here will be on the length of time it takes the checker to return
    if a guess is correct. It will be slower based on how many characters in the guess are correct.

    Some tips for implementing and running a timing attack:

    - You will need to do many trials of each operation in order to get a large enough sample size to
        have a measurable time difference. The time can be impacted by OS scheduling, cache effects,
        imprecision in measurement, and other issues. For that reason, I had successes only when running
        ~1_000_000 trials if using a debug build, and ~10_000_000 trials when using a release build.
    - You can measure times in rust by using `let now = Instant::now()` before starting the block of
        code to measured, and `now.elapsed().as_nanos()` after the block. This will give the number of
        nanoseconds elapsed as a u128.
    - You can make your tests run much, much faster even with more trials by running it in release mode.
        Use the command `cargo test --release --package pba-assignment --features run-slow-tests --lib -- p9_attacks::crack_password_test --exact --nocapture`
        However, if you do this, you need to make sure that the compiler doesn't compile out the inner
        loop that is timed.  This can be circumvented by making it so it cannot be ignored, by using the
        below as your inner loop code.
        ```
        let corr = checker.is_correct(&guess);
        if corr {
            return {whatever you return from your function};
        }
        ```
    - Including print statements that show the progress of the cracking can be helpful if you're not sure why it failed.
    - Consider if the code will still take a different amount of time for the last character! If not,
        you might need to take a different approach for the last character.
    - If you are having issues where a specific character takes an unexpectedly long time when testing,
        and ruins your password cracking, you can try having a "runoff" to correct for one-off effects like
        that. In order to take this approach, track the best 2 candidates when iterating over possible
        characters, and test them again at the end, choosing the better of the two. This will not fix
        _every_ failure, but it should fix most.

    In total, the staff solution running on an M1 pro chip can crack a 15-character password in about 25
    seconds, using 10 million trials per character guessed. It can crack a 7 character password in 6
    seconds.
    */

    /// Return the length of the password stored inside the insecure password checker
    pub fn crack_len(checker: &InsecurePasswordChecker) -> usize {
        let mut times = Vec::new();
        for i in 1..=MAX_PASSWORD_LEN {
            let guess = vec![0; i];
            let now = Instant::now();
            for _ in 0..10_000_000 {
                checker.is_correct(&guess);
            }
            let elapsed = now.elapsed().as_nanos();
            times.push(elapsed);
            if i > 1 && elapsed > times[..i - 1].iter().sum::<u128>() / (i as u128 - 1) * 2 {
                return i;
            }
        }
        MAX_PASSWORD_LEN
    }

    pub fn crack_password(checker: &InsecurePasswordChecker) -> Vec<u8> {
        let pass_len = crack_len(checker);
        let mut guess = vec![0; pass_len];
        for i in 0..pass_len {
            let mut max_time = 0;
            let mut next_char = 0;
            for c in alphanumeric_iterator() {
                guess[i] = c;
                let now = Instant::now();
                for _ in 0..10_000_000 {
                    checker.is_correct(&guess);
                }
                let elapsed = now.elapsed().as_nanos();
                if elapsed > max_time {
                    max_time = elapsed;
                    next_char = c;
                }
            }
            guess[i] = next_char;
        }
        guess
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn crack_length_short_test() {
        let password = b"hunter2".to_vec();
        let checker = InsecurePasswordChecker { password };

        let size = crack_len(&checker);
        assert_eq!(7, size);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn crack_length_long_test() {
        let password = b"alongpassword27".to_vec();
        let checker = InsecurePasswordChecker { password };

        let size = crack_len(&checker);
        assert_eq!(15, size);
    }

    #[test]
    #[cfg_attr(
        any(not(feature = "slow-tests"), not(feature = "optional-tests")),
        ignore
    )]
    fn crack_password_test() {
        let password = b"hunter2".to_vec();
        let checker = InsecurePasswordChecker {
            password: password.clone(),
        };

        let pass = crack_password(&checker);
        assert_eq!(password, pass);
    }

    // This test takes a long time to run with cargo test, so we skip it there
    #[test]
    #[cfg_attr(
        any(not(feature = "slow-tests"), not(feature = "optional-tests")),
        ignore
    )]
    fn crack_password_long_test() {
        let password = b"alongpassword27".to_vec();
        let checker = InsecurePasswordChecker {
            password: password.clone(),
        };

        let pass = crack_password(&checker);
        assert_eq!(password, pass);
    }
}

/// This function is not graded. It is just for collecting feedback.
/// On a scale from 0 - 100, with zero being extremely easy and 100 being extremely hard, how hard
/// did you find the exercises in this file?
pub fn how_hard_was_this_section() -> u8 {
    80
}

/// This function is not graded. It is just for collecting feedback.
/// About how much time (in hours) did you spend on the exercises in this file?
pub fn how_many_hours_did_you_spend_on_this_section() -> f32 {
    4.0
}
