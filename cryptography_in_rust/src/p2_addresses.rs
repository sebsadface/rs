#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use core::panic;

use byteorder::{LittleEndian, ReadBytesExt};
use hex::ToHex;
use phf::phf_map;
use sp_core::ecdsa::{Pair as ECDSAPair, Public as ECDSAPublic};
use sp_core::ed25519::{Pair as Ed25519Pair, Public as Ed25519Public};
use sp_core::sr25519::{Pair as Sr25519Pair, Public as Sr25519Public};
use sp_core::{
    crypto::{Derive, DeriveJunction, Ss58Codec},
    Pair, Public,
};

/// A seed phrase for a key according to the BIP39 standard
const BIP39_STR: &str = "source upgrade van toy cross smooth write erupt uncover today injury say wealth silk thought slide shadow comfort hazard planet wisdom problem review pudding";

/// The SS58 address corresponding to the above seed phrase
const SS58_ADDRESS: &str = "5GEkFD1WxzmfasT7yMUERDprkEueFEDrSojE3ajwxXvfYYaF";

// For the following functions, it will be helpful to figure out how to do them using both code
// and the subkey tool. Additionally, reading through the subkey documentation may be helpful.
// The documentation is here: https://docs.substrate.io/reference/command-line-tools/subkey/
// with installation instructions: https://docs.substrate.io/reference/command-line-tools/subkey/#installation

/// Generate the sr25519 keypair corresponding to the const bip39 phrase
pub fn generate_sr25519_pair() -> Sr25519Pair {
    match Sr25519Pair::from_string(BIP39_STR, None) {
        Ok(pair) => pair,
        Err(_) => panic!("generate_sr25519_pair on {}.", BIP39_STR),
    }
}

/// Generate the ed25519 keypair corresponding to the const bip39 phrase
pub fn generate_ed25519_pair() -> Ed25519Pair {
    match Ed25519Pair::from_string(BIP39_STR, None) {
        Ok(pair) => pair,
        Err(_) => panic!("generate_ed25519_pair on {}.", BIP39_STR),
    }
}

/// Generate the ecdsa keypair corresponding to the const bip39 phrase
pub fn generate_ecdsa_pair() -> ECDSAPair {
    match ECDSAPair::from_string(BIP39_STR, None) {
        Ok(pair) => pair,
        Err(_) => panic!("generate_ecdsa_pair failed on {}.", BIP39_STR),
    }
}

/// Generate a child keypair of the sr25519 keypair, with the derivation path "children" and
/// "0", where "children" is a hard derivation, and "0" is soft.
pub fn generate_derived_addresses_from_sr25519_pair() -> Sr25519Pair {
    match Sr25519Pair::from_string_with_seed([BIP39_STR, "//children/0"].concat().as_str(), None) {
        Ok((pair, _)) => pair,
        Err(error) => {
            panic!(
                "generate_derived_addresses_from_sr25519_pair failed with {}.",
                error
            )
        }
    }
}

/// Generate a child keypair corresponding to the address passed in. The address is provided in
/// SS58 format, and the derivation path should be "test_derivation" and "5", with both being
/// soft derivations.
pub fn generate_derived_public_from_address(address: &str) -> Sr25519Public {
    match Sr25519Public::from_string([address, "/test_derivation/5"].concat().as_str()) {
        Ok(pub_key) => pub_key,
        Err(error) => panic!("generate_derived_public_from_address failed with {}", error),
    }
}

/// Generate the substrate test pair corresponding to Alice in sr25519
pub fn alice_sr25519() -> Sr25519Pair {
    match Sr25519Pair::from_string("//Alice", None) {
        Ok(test_pair) => test_pair,
        Err(error) => panic!("alice_sr25519 failed with {}.", error),
    }
}

/// Generate the substrate test pair corresponding to Alice in ECDSA
pub fn alice_ecdsa() -> ECDSAPair {
    match ECDSAPair::from_string("//Alice", None) {
        Ok(test_pair) => test_pair,
        Err(error) => panic!("alice_ecdsa failed with {}.", error),
    }
}

/// Generate the sr25519 keypair corresponding to the const bip39 phrase using the password
/// 'hunter2'
pub fn generate_with_password() -> Sr25519Pair {
    match Sr25519Pair::from_string(BIP39_STR, Some("hunter2")) {
        Ok(pair) => pair,
        Err(error) => panic!("generate_with_password failed with {}", error),
    }
}

// Now that we have some familiarity with seeds, phrases, and password derivation, let's look a
// little into how seed phrases actually work! BIP39 uses a fixed english dictionary, and maps
// those words into specific bit sequences. BIP39 is designed to represent entropy in a
// human-readable form that is easy to remember.
//
// It also includes a checksum, so if you change one word in a seed phrase it won't work. Feel free
// to try it yourself! Generate a seed phrase with `subkey generate` and then try to change one word
// and inspect it with `subkey inspect $my_changed_phrase`.
//
// For this exercise, we will make our own tiny version of a seed phrase generator. We will only
// support 4-byte seeds, and have 16 possible words.
#[derive(Debug, PartialEq, Eq)]
pub struct TinySeed(u32);

/// This is a mapping from the 16 words in our dictionary to the hex character it represents.
/// We only have 16 words, so each word corresponds to a hex character, or 4 bits. BIP39 has
/// 2048 words, so each word is 11 bits.
///
/// We will convert from words to bytes in the following fashion:
///     - Convert the list of words into their corresponding hex characters
///     - Interpret the list of hex characters as little-endian bytes
///
/// Hints:
///     - The `hex` crate provides nice functionality for hex encoding/decoding.
///     - `char::to_digit` might be useful.
static WORDS_TO_ENTROPY: phf::Map<&'static str, char> = phf_map! {
    "polkadot" => '0',
    "blockchain" => '1',
    "academy" => '2',
    "berkeley" => '3',
    "chancellor" => '4',
    "on" => '5',
    "brink" => '6',
    "of" => '7',
    "second" => '8',
    "bailout" => '9',
    "for" => 'a',
    "banks" => 'b',
    "not" => 'c',
    "your" => 'd',
    "keys" => 'e',
    "crypto" => 'f',
};

/// This is an, where the ith entry is the word representing i in the dictionary.
static ENTROPY_TO_WORDS: [&str; 16] = [
    "polkadot",
    "blockchain",
    "academy",
    "berkeley",
    "chancellor",
    "on",
    "brink",
    "of",
    "second",
    "bailout",
    "for",
    "banks",
    "not",
    "your",
    "keys",
    "crypto",
];

/// Convert a tiny seed to a phrase, based on the u32 interpreted as little endian bytes
pub fn seed_to_phrase(seed: TinySeed) -> String {
    let bytes = seed.0.to_le_bytes();
    bytes
        .iter()
        .map(|&byte| {
            let first = ENTROPY_TO_WORDS[(byte >> 4) as usize];
            let second = ENTROPY_TO_WORDS[(byte & 0x0F) as usize];
            format!("{} {}", first, second)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a phrase to a tiny seed. Errors if any words are not in the wordlist, or there is
/// the wrong number of words. This function should never panic.
pub fn phrase_to_seed(phrase: &str) -> Result<TinySeed, ()> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() % 2 != 0 {
        return Err(());
    }
    let mut bytes = vec![];
    for chunk in words.chunks(2) {
        let first = WORDS_TO_ENTROPY.get(chunk[0]).ok_or(())?;
        let second = WORDS_TO_ENTROPY.get(chunk[1]).ok_or(())?;
        let byte = (first.to_digit(16).unwrap() << 4) | second.to_digit(16).unwrap();
        bytes.push(byte as u8);
    }
    Ok(TinySeed(u32::from_le_bytes(bytes.try_into().unwrap())))
}

/// A trucated hash function over a u32. We only use 1 byte of the hash value as a the checksum
/// for our tiny seed conversions.
pub fn truncated_hash_u32(x: u32) -> u8 {
    sp_core::blake2_128(x.to_le_bytes().as_slice())[0]
}

/// Convert a tiny seed to a phrase, based on the u32 interpreted as little endian bytes. We
/// also append a 1 byte checksum to the end of the phrase, also in phrase form. The resulting
/// phrase should be 10 words long. The checksum should come from the function `truncated_hash_u32`.
pub fn seed_to_phrase_with_checksum(seed: TinySeed) -> String {
    let checksum = truncated_hash_u32(seed.0);
    let mut phrase = seed_to_phrase(seed);
    phrase.push_str(" ");
    phrase.push_str(ENTROPY_TO_WORDS[(checksum >> 4) as usize]);
    phrase.push_str(" ");
    phrase.push_str(ENTROPY_TO_WORDS[(checksum & 0x0F) as usize]);
    phrase
}

/// Convert a phrase which includes a checksum to a tiny seed. Errors if any words are not in
/// the wordlist, there is the wrong number of words, or the checksum is wrong.
///
/// This function should never panic.
pub fn phrase_to_seed_with_checksum(phrase: &str) -> Result<TinySeed, ()> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() != 10 {
        return Err(());
    }
    let checksum_words = &words[words.len() - 2..];
    let checksum = (WORDS_TO_ENTROPY
        .get(checksum_words[0])
        .ok_or(())?
        .to_digit(16)
        .unwrap()
        << 4)
        | WORDS_TO_ENTROPY
            .get(checksum_words[1])
            .ok_or(())?
            .to_digit(16)
            .unwrap();
    let seed = phrase_to_seed(&words[..words.len() - 2].join(" "))?;
    if truncated_hash_u32(seed.0) != checksum as u8 {
        return Err(());
    }
    Ok(seed)
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
    3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sr25519_pair_test() {
        assert_eq!(
            "5GEkFD1WxzmfasT7yMUERDprkEueFEDrSojE3ajwxXvfYYaF".to_string(),
            generate_sr25519_pair().public().to_ss58check()
        )
    }

    #[test]
    fn generate_ed25519_pair_test() {
        // You might expect that an ed25519 key from the same seed as an sr25519 key would be the
        // same, because they use the same underlying curve (hence the names). However, that is not
        // necessarily the case! Feel free to dig into the code to see where they differ. If you
        // can't find out where, or are confused by how to see, call over an instructor!
        assert_eq!(
            "5CViXS31EkSxFgY7c3PcncsM2TbmqnxjXoLLVuMJyqP6PTGp".to_string(),
            generate_ed25519_pair().public().to_ss58check()
        )
    }

    #[test]
    fn generate_ecdsa_pair_test() {
        assert_eq!(
            "KW5diveAeLbPDuQWMeped6kA7wbtzvX5hoH1ocLKhPwx2PkCZ".to_string(),
            generate_ecdsa_pair().public().to_ss58check()
        )
    }

    #[test]
    fn generate_derived_addresses_from_sr25519_pair_test() {
        assert_eq!(
            "5CFFJoP6vZokcgdcQWp8XvQ7FQMMrb9mYgJF32zVbHmQs895".to_string(),
            generate_derived_addresses_from_sr25519_pair()
                .public()
                .to_ss58check()
        )
    }

    #[test]
    fn generate_derived_public_from_address_test() {
        assert_eq!(
            "5GgZq5hswE9s7tEpnimAFcw3PFgVUvWHQAxHqVisdhLQbKPJ".to_string(),
            generate_derived_public_from_address(SS58_ADDRESS).to_ss58check(),
        )
    }

    #[test]
    fn alice_sr25519_test() {
        assert_eq!(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            alice_sr25519().public().to_ss58check(),
        )
    }

    #[test]
    fn alice_ecdsa_test() {
        assert_eq!(
            "KW39r9CJjAVzmkf9zQ4YDb2hqfAVGdRqn53eRqyruqpxAP5YL".to_string(),
            alice_ecdsa().public().to_ss58check(),
        )
    }

    #[test]
    fn generate_with_password_test() {
        assert_eq!(
            "5CrSG9W4XppN2CEENE6UMQcBXHBiMsyKgkH3Qqmr2aa39UXx".to_string(),
            generate_with_password().public().to_ss58check(),
        )
    }
}

#[cfg(test)]
mod optional_tests {
    use super::*;
    // If you're having trouble with these tests, make sure to check the endianness of your integers
    // and decoding! It can often be challenging to work with. The functions u32::from_le_bytes and
    // u32::to_le_bytes may be helpful.

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn seed_to_phrase_test() {
        let seed = TinySeed(762150966);
        let phrase = seed_to_phrase(seed);
        assert_eq!(
            "berkeley brink second polkadot brink your academy your",
            &phrase
        );
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn phrase_to_seed_test() {
        let phrase = "chancellor of not your keys not your crypto";
        let seed = phrase_to_seed(phrase).unwrap();
        assert_eq!(TinySeed(3756838215), seed);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn tiny_seed_return_trips() {
        // If you're having trouble with the roundtrip,
        let seed = TinySeed(762150966);
        let phrase = seed_to_phrase(seed);
        assert_eq!(
            "berkeley brink second polkadot brink your academy your",
            &phrase
        );
        assert_eq!(TinySeed(762150966), phrase_to_seed(&phrase).unwrap());

        let phrase = "chancellor of not your keys not your crypto";
        let seed = phrase_to_seed(phrase).unwrap();
        assert_eq!(TinySeed(3756838215), seed);
        assert_eq!(phrase.to_string(), seed_to_phrase(seed));
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn seed_to_phrase_checksum_test() {
        let seed = TinySeed(762150966);
        let phrase = seed_to_phrase_with_checksum(seed);
        assert_eq!(
            "berkeley brink second polkadot brink your academy your academy brink",
            &phrase
        );
        assert_eq!(
            TinySeed(762150966),
            phrase_to_seed_with_checksum(&phrase).unwrap()
        );
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn phrase_to_seed_checksum_test() {
        let phrase = "chancellor of not your keys not your crypto chancellor crypto";
        let seed = phrase_to_seed_with_checksum(phrase).unwrap();
        assert_eq!(TinySeed(3756838215), seed);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn tiny_seed_checksum_return_trips() {
        let seed = TinySeed(762150966);
        let phrase = seed_to_phrase_with_checksum(seed);
        assert_eq!(
            "berkeley brink second polkadot brink your academy your academy brink",
            &phrase
        );
        assert_eq!(
            TinySeed(762150966),
            phrase_to_seed_with_checksum(&phrase).unwrap()
        );

        let phrase = "chancellor of not your keys not your crypto chancellor crypto";
        let seed = phrase_to_seed_with_checksum(phrase).unwrap();
        assert_eq!(TinySeed(3756838215), seed);
        assert_eq!(phrase.to_string(), seed_to_phrase_with_checksum(seed));
    }
}
