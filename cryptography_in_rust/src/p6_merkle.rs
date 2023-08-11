//! Write a Merkle tree implementation that supports proofs and multiproofs.

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use bip32::secp256k1::elliptic_curve::subtle::ConstantTimeEq;
use hex::encode;
use rand::SeedableRng;
use rand_core::block;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

/// We'll use Rust's built-in hashing which returns a u64 type.
/// This alias just helps us understand when we're treating the number as a hash
pub type HashValue = u64;

/// Helper function that makes the hashing interface easier to understand.
pub fn hash<T: Hash>(t: &T) -> HashValue {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Given a vector of data blocks this function adds padding blocks to the end
/// until the length is a power of two which is needed for Merkle trees.
/// The padding value should be the empty string "".
pub fn pad_base_layer(blocks: &mut Vec<&str>) {
    while (blocks.len() & (blocks.len() - 1)) != 0 {
        blocks.push("");
    }
}

/// Helper function to combine two hashes and compute the hash of the combination.
/// This will be useful when building the intermediate nodes in the Merkle tree.
///
/// Our implementation will hex-encode the hashes (as little-endian uints) into strings, concatenate
/// the strings, and then hash that string.
pub fn concatenate_hash_values(left: HashValue, right: HashValue) -> HashValue {
    hash(&(encode(left.to_le_bytes()) + &encode(right.to_le_bytes())))
}

/// Calculates the Merkle root of a sentence. We consider each word in the sentence to
/// be one block. Words are separated by one or more spaces.
///
/// Example:
/// Sentence: "You trust me, right?"
/// "You", "trust", "me," "right?"
/// Notice that the punctuation like the comma and exclamation point are included in the words
/// but the spaces are not.
pub fn calculate_merkle_root(sentence: &str) -> HashValue {
    let mut blocks = sentence.trim().split_whitespace().collect();
    pad_base_layer(&mut blocks);
    let mut hashes = Vec::new();
    for b in blocks {
        hashes.push(hash(&b))
    }

    while hashes.len() > 1 {
        hashes = (0..hashes.len())
            .step_by(2)
            .map(|i| concatenate_hash_values(hashes[i], hashes[i + 1]))
            .collect();
    }

    hashes[0]
}

/// A representation of a sibling node along the Merkle path from the data
/// to the root. It is necessary to specify which side the sibling is on
/// so that the hash values can be combined in the same order.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SiblingNode {
    Left(HashValue),
    Right(HashValue),
}

/// A proof is just an alias for a vec of sibling nodes.
pub type MerkleProof = Vec<SiblingNode>;

/// Generates a Merkle proof that one particular word is contained
/// in the given sentence. You provide the sentence and the index of the word
/// which you want a proof.
///
/// Panics if the index is beyond the length of the sentence.
///
/// Example: I want to prove that the word "trust" is in the sentence "You trust me, right?"
/// So I call generate_proof("You trust me, right?", 1)
/// And I get back the merkle root and list of intermediate nodes from which the
/// root can be reconstructed.
pub fn generate_proof(sentence: &str, index: usize) -> (HashValue, MerkleProof) {
    let mut blocks: Vec<&str> = sentence.trim().split_whitespace().collect();
    if index >= blocks.len() {
        panic!("Index is beyond the length of the sentence.")
    }
    pad_base_layer(&mut blocks);
    let mut hashes = Vec::new();
    for b in blocks {
        hashes.push(hash(&b));
    }

    let mut merkle_proof = vec![{
        if index % 2 == 0 {
            SiblingNode::Right(hashes[index + 1].clone())
        } else {
            SiblingNode::Left(hashes[index - 1].clone())
        }
    }];

    let mut prev_index = index;
    loop {
        hashes = (0..hashes.len())
            .step_by(2)
            .map(|i| concatenate_hash_values(hashes[i], hashes[i + 1]))
            .collect();
        if hashes.len() <= 1 {
            break;
        }
        prev_index = prev_index / 2;
        merkle_proof.push(if prev_index % 2 == 0 {
            SiblingNode::Right(hashes[prev_index + 1].clone())
        } else {
            SiblingNode::Left(hashes[prev_index - 1].clone())
        });
    }

    (hashes[0], merkle_proof)
}

/// Checks whether the given word is contained in a sentence, without knowing the whole sentence.
/// Rather we only know the merkle root of the sentence and a proof.
pub fn validate_proof(root: &HashValue, word: &str, proof: MerkleProof) -> bool {
    let mut actual_root = hash(&word);

    for sibling in proof {
        match sibling {
            SiblingNode::Left(n) => actual_root = concatenate_hash_values(n, actual_root),
            SiblingNode::Right(n) => actual_root = concatenate_hash_values(actual_root, n),
        };
    }

    *root == actual_root
}

/// A compact Merkle multiproof is used to prove multiple entries in a Merkle tree in a highly
/// space-efficient manner.
#[derive(Debug, PartialEq, Eq)]
pub struct CompactMerkleMultiProof {
    // The indices requested in the initial proof generation
    pub leaf_indices: Vec<usize>,
    // The additional hashes necessary for computing the proof, given in order from
    // lower to higher index, lower in the tree to higher in the tree.
    pub hashes: Vec<HashValue>,
}

/// Generate a compact multiproof that some words are contained in the given sentence. Returns the
/// root of the merkle tree, and the compact multiproof. You provide the words at `indices` in the
/// same order as within `indices` to verify the proof. `indices` is not necessarily sorted.
///
/// Panics if any index is beyond the length of the sentence, or any index is duplicated.
///
/// ## Explanation
///
/// To understand the compaction in a multiproof, see the following merkle tree. To verify a proof
/// for the X's, only the entries marked with H are necessary. The rest can be calculated. Then, the
/// hashes necessary are ordered based on the access order. The H's in the merkle tree are marked
/// with their index in the output compact proof.
///
/// ```text
///                                      O            
///                                   /     \           
///                                O           O     
///                              /   \       /   \     
///                             O    H_1   H_2    O  
///                            / \   / \   / \   / \
///                           X  X  O  O  O  O  X  H_0
/// ```
///
/// The proof generation process would proceed similarly to a normal merkle proof generation, but we
/// need to keep track of which hashes are known to the verifier by a certain height, and which need
/// to be given to them.
///
/// In the leaf-node layer, the first pair of hashes are both
/// known, and so no extra data is needed to go up the tree.  In the next two pairs of hashes,
/// neither are known, and so the verifier does not need them. In the last set, the verifier only
/// knows the left hash, and so the right hash must be provided.
///
/// In the second layer, the first and fourth hashes are known. The first pair is missing the right
/// hash, which must be included in the proof. The second pair is missing the left hash, which also
/// must be included.
///
/// In the final layer before the root, both hashes are known to the verifier, and so no further
/// proof is needed.
///
/// The final proof for this example would be
/// ```ignore
/// CompactMerkleMultiProof {
///     leaf_indices: [0, 1, 6],
///     hashes: [H_0, H_1, H_2]
/// }
/// ```
pub fn generate_compact_multiproof(
    sentence: &str,
    indices: Vec<usize>,
) -> (HashValue, CompactMerkleMultiProof) {
    let mut blocks: Vec<&str> = sentence.trim().split_whitespace().collect();

    let mut leaf_indicies = Vec::new();

    // check for invalid indices
    for i in indices {
        if i >= blocks.len() {
            panic!("Index is beyond the length of the sentence.");
        }

        if leaf_indicies.contains(&i) {
            panic!("Index {} is duplicated.", i);
        }

        leaf_indicies.push(i);
    }

    pad_base_layer(&mut blocks);
    let mut hashes = Vec::new();
    for b in blocks {
        hashes.push(hash(&b));
    }

    let mut merkle_hashes = Vec::new();

    for i in &leaf_indicies {
        if i % 2 == 0 && !leaf_indicies.contains(&(i + 1)) {
            merkle_hashes.push(hashes[i + 1].clone())
        } else if *i > 0 && !leaf_indicies.contains(&(i - 1)) {
            merkle_hashes.push(hashes[i - 1].clone())
        }
    }

    let mut prev_indices = leaf_indicies.clone();
    loop {
        hashes = (0..hashes.len())
            .step_by(2)
            .map(|i| concatenate_hash_values(hashes[i], hashes[i + 1]))
            .collect();
        if hashes.len() <= 1 {
            break;
        }

        let mut intermediate = Vec::new();
        for i in &prev_indices {
            if !intermediate.contains(&(i / 2)) {
                intermediate.push(i / 2);
            }
        }
        prev_indices = intermediate.clone();

        for i in &prev_indices {
            if i % 2 == 0 && !prev_indices.contains(&(i + 1)) {
                merkle_hashes.push(hashes[i + 1].clone());
            } else if *i > 0 && !prev_indices.contains(&(i - 1)) {
                merkle_hashes.push(hashes[i - 1].clone());
            }
        }
    }

    (
        hashes[0],
        CompactMerkleMultiProof {
            leaf_indices: leaf_indicies,
            hashes: merkle_hashes,
        },
    )
}

/// Validate a compact merkle multiproof to check whether a list of words is contained in a sentence, based on the merkle root of the sentence.
/// The words must be in the same order as the indices passed in to generate the multiproof.
/// Duplicate indices in the proof are rejected by returning false.
pub fn validate_compact_multiproof(
    root: &HashValue,
    words: Vec<&str>,
    proof: CompactMerkleMultiProof,
) -> bool {
    // check invalid indicies
    let mut dup = proof.leaf_indices.clone();
    dup.sort();
    for i in 1..dup.len() {
        if dup[i] == dup[i - 1] {
            return false; //found dupliucate
        }
    }

    let mut proof_hashes = proof.hashes.clone();
    let mut parent_hashes = Vec::new();
    let mut next_indices = proof.leaf_indices.clone();

    for w in 0..words.len() {
        if next_indices[w] % 2 == 0 {
            parent_hashes.push(hash(&words[w]));
            if w + 1 >= next_indices.len() || next_indices[w] + 1 != next_indices[w + 1] {
                parent_hashes.push(proof_hashes.remove(0));
            }
        } else {
            if w <= 0 || next_indices[w] - 1 != next_indices[w - 1] {
                parent_hashes.push(proof_hashes.remove(0));
            }
            parent_hashes.push(hash(&words[w]));
        }
    }

    while !proof_hashes.is_empty() {
        parent_hashes = (0..parent_hashes.len())
            .step_by(2)
            .map(|i| concatenate_hash_values(parent_hashes[i], parent_hashes[i + 1]))
            .collect();

        let mut intermediate = Vec::new();
        for i in &next_indices {
            if !intermediate.contains(&(i / 2)) {
                intermediate.push(i / 2);
            }
        }
        next_indices = intermediate.clone();

        let mut temp = Vec::new();

        for w in 0..parent_hashes.len() {
            if next_indices[w] % 2 == 0 {
                temp.push(parent_hashes[w].clone());
                if w + 1 >= next_indices.len() || next_indices[w] + 1 != next_indices[w + 1] {
                    temp.push(proof_hashes.remove(0));
                }
            } else {
                if w <= 0 || next_indices[w] - 1 != next_indices[w - 1] {
                    temp.push(proof_hashes.remove(0));
                }
                temp.push(parent_hashes[w].clone());
            }
        }
        parent_hashes = temp.clone();
    }

    //get the root
    while parent_hashes.len() > 1 {
        parent_hashes = (0..parent_hashes.len())
            .step_by(2)
            .map(|i| concatenate_hash_values(parent_hashes[i], parent_hashes[i + 1]))
            .collect();
    }
    parent_hashes[0] == *root
}

// Now that we have a normal and compact method to generate proofs, let's compare how
// space-efficient the two are. The two functions below will be helpful for answering the questions
// in the readme.

/// Generate a space-separated string of `n` random 4-letter words. Use of this function is not
/// mandatory.
pub fn string_of_random_words(n: usize) -> String {
    let mut ret = String::new();
    for i in 0..n {
        ret.push_str(random_word::gen_len(4).unwrap());
        if i != n - 1 {
            ret.push(' ');
        }
    }
    ret
}

/// Given a string of words, and the length of the words from which to generate proofs, generate
/// proofs for `num_proofs` random indices in `[0, length)`.  Uses `rng_seed` as the rng seed, if
/// replicability is desired.
///
/// Return the size of the compact multiproof, and then the combined size of the standard merkle proofs.
///
/// This function assumes the proof generation is correct, and does not validate them.
pub fn compare_proof_sizes(
    words: &str,
    length: usize,
    num_proofs: usize,
    rng_seed: u64,
) -> (usize, usize) {
    assert!(
        num_proofs <= length,
        "Cannot make more proofs than available indices!"
    );

    let mut rng = rand::rngs::SmallRng::seed_from_u64(rng_seed);
    let indices = rand::seq::index::sample(&mut rng, length, num_proofs).into_vec();
    let (_, compact_proof) = generate_compact_multiproof(words, indices.clone());
    // Manually calculate memory sizes
    let compact_size = mem::size_of::<usize>() * compact_proof.leaf_indices.len()
        + mem::size_of::<HashValue>() * compact_proof.hashes.len()
        + mem::size_of::<Vec<usize>>() * 2;

    let mut individual_size = 0;
    for i in indices {
        let (_, proof) = generate_proof(words, i);
        individual_size +=
            mem::size_of::<Vec<usize>>() + mem::size_of::<SiblingNode>() * proof.len();
    }

    (compact_size, individual_size)
}

#[test]
// #[ignore]
fn student_test_to_compare_sizes() {
    // Maybe write a test here to compare proof sizes in order to get answers to the following
    // questions.
    let comparison = compare_proof_sizes(&string_of_random_words(2023), 1000, 326, 0);
    println!("compact: {}, individual: {}", comparison.0, comparison.1);
    assert!(10 * comparison.0 <= comparison.1);
}

/// An answer to the below short answer problems
#[derive(PartialEq, Debug)]
pub struct ShortAnswer {
    /// The answer to the problem
    pub answer: usize,
    /// The explanation associated with an answer. This should be 1-3 sentences. No need to make it
    /// too long!
    pub explanation: String,
}

// For the following two problems, you will need to make use of the `compare_proof_sizes` function
// defined above. Writing a test to check different values can be helpful. Additionally, running
// your test in release mode will probably speed up your code by approximately 10x. If your test is
// named `my_test`, you can run the test in release mode with the command:
// `cargo test --release --package pba-assignment --lib -- p6_merkle::my_test --exact --nocapture`
//
// The `explanation` field of the returned answer should be a 1-3 sentence explanation of how you
// arrived at the answer you did.

/// Given a merkle tree with exactly 2023 items, what is the breakpoint B for the number of proofs
/// where a compact Merkle multiproof is almost exactly 10x as space-efficient as distinct single
/// Merkle proofs? In other words, if you request more than B proofs, it will be more than 10x as
/// space efficient, and if you request less than B proofs, it will be less than 10x as space
/// efficient.
pub fn short_answer_1() -> ShortAnswer {
    ShortAnswer {
        answer: 644,
        explanation: String::from(
            "
            I stated with 100 proofs and keep running the test with increasing number of proofs and finally when I reached 644 proofs, 
            the size of the compact multiproof is 12880 which is almost exactly 10x as space-efficient as distinct single Merkle proofs (128800).
            When the number of proofs is greater than 644, the compact multiproof is more than 10x as space-efficient as individual Merkle proofs.
        ",
        ),
    }
}

/// Given a merkle tree with exactly 2023 items where the proofs are only about the first 1000
/// items, what is the breakpoint B for the number of proofs where a compact Merkle multiproof is
/// almost exactly 10x as space-efficient as distinct single Merkle proofs? In other words, if you
/// request more than B proofs, it will be more than 10x as space efficient, and if you request less
/// than B proofs, it will be less than 10x as space efficient.
///
/// Hint: You can set `length` to 1000 in `compare_proof_sizes` in order to simulate this.
pub fn short_answer_2() -> ShortAnswer {
    ShortAnswer {
        answer: 326,
        explanation: String::from(
            "
            I stated with 100 proofs and keep running the test with increasing number of proofs and finally when I reached 326 proofs,
            the size of the compact multiproof is 6520 which is almost exactly 10x as space-efficient as distinct single Merkle proofs (65200).
            When the number of proofs is greater than 326, the compact multiproof is more than 10x as space-efficient as individual Merkle proofs.
        ",
        ),
    }
}

/// This function is not graded. It is just for collecting feedback.
/// On a scale from 0 - 100, with zero being extremely easy and 100 being extremely hard, how hard
/// did you find the exercises in this section?
pub fn how_hard_was_this_section() -> u8 {
    80
}

/// This function is not graded. It is just for collecting feedback.
/// About how much time (in hours) did you spend on the exercises in this section?
pub fn how_many_hours_did_you_spend_on_this_section() -> f32 {
    5.8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_base_layer_sanity_check() {
        let mut data = vec!["a", "b", "c"];
        let expected = vec!["a", "b", "c", ""];
        pad_base_layer(&mut data);
        assert_eq!(expected, data);
    }

    #[test]
    fn concatenate_hash_values_sanity_check() {
        let left = hash(&"a");
        let right = hash(&"b");
        assert_eq!(13491948173500414413, concatenate_hash_values(left, right));
    }

    #[test]
    fn calculate_merkle_root_sanity_check() {
        let sentence = "You trust me, right?";
        assert_eq!(4373588283528574023, calculate_merkle_root(sentence));
    }

    #[test]
    fn proof_generation_sanity_check() {
        let sentence = "You trust me, right?";
        let expected = (
            4373588283528574023,
            vec![
                SiblingNode::Left(4099928055547683737),
                SiblingNode::Right(2769272874327709143),
            ],
        );
        assert_eq!(expected, generate_proof(sentence, 1));
    }

    #[test]
    fn validate_proof_sanity_check() {
        let word = "trust";
        let root = 4373588283528574023;
        let proof = vec![
            SiblingNode::Left(4099928055547683737),
            SiblingNode::Right(2769272874327709143),
        ];
        assert!(validate_proof(&root, word, proof));
    }

    #[test]
    fn generate_compact_multiproof_sanity_check() {
        let sentence = "Here's an eight word sentence, special for you.";
        let indices = vec![0, 1, 6];
        let expected = (
            14965309246218747603,
            CompactMerkleMultiProof {
                leaf_indices: vec![0, 1, 6],
                hashes: vec![
                    1513025021886310739,
                    7640678380001893133,
                    5879108026335697459,
                ],
            },
        );
        assert_eq!(expected, generate_compact_multiproof(sentence, indices));
    }

    #[test]
    fn validate_compact_multiproof_sanity_check() {
        let proof = (
            14965309246218747603u64,
            CompactMerkleMultiProof {
                leaf_indices: vec![0, 1, 6],
                hashes: vec![
                    1513025021886310739,
                    7640678380001893133,
                    5879108026335697459,
                ],
            },
        );
        let words = vec!["Here's", "an", "for"];
        assert_eq!(true, validate_compact_multiproof(&proof.0, words, proof.1));
    }

    #[test]
    fn short_answer_1_has_answer() {
        assert_ne!(
            ShortAnswer {
                answer: 0,
                explanation: "".to_string(),
            },
            short_answer_1()
        )
    }

    #[test]
    fn short_answer_2_has_answer() {
        assert_ne!(
            ShortAnswer {
                answer: 0,
                explanation: "".to_string(),
            },
            short_answer_2()
        )
    }

    #[test]
    fn calculate_merkle_root_sanity_check_2() {
        let sentence = "You trust me?";
        assert_eq!(8656240816105094750, calculate_merkle_root(sentence));
    }
}
