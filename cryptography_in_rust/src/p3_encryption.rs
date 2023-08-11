#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use aes::cipher::generic_array::{typenum::U12, ArrayLength, GenericArray};
use aes::{self, Aes128};
use aes_gcm_siv::aead::{Aead, Payload};
use aes_gcm_siv::{AeadInPlace, Aes128GcmSiv, KeyInit, Nonce};
use hex::ToHex;
use rand::{self, Fill, Rng};

const BASE_MESSAGE: &[u8] = b"I'm encrypting this at PBA Berkeley 2023!";

/// A randomly generated 128-bit key. In practice, you'll want to use a good RNG to generate
/// this.
///
/// Trust me, it's random. I generated it myself!
const SYM_KEY: &[u8; 16] = &[
    6, 108, 74, 203, 170, 212, 94, 238, 171, 104, 19, 17, 248, 197, 127, 138,
];

/// A struct that represents an encrypted message which is encrypted with AES128-GCM-SIV.
///
/// In general, you only need to make sure you encrypt and decrypt a message with the same scheme.
/// However, for the sake of learning, let's deconstruct what that scheme actually means and
/// explain why it is a good choice. Like most cryptographies, this is formalized in an RFC. This
/// one is RFC 8452.
/// - **AES128**: This uses the 128-bit version of the Advanced Encryption Standard (AES) block
///     cipher. This is a standard encryption algorithm that is well-regarded.
/// - **GCM**: This uses the Galois Counter Mode in order to provide authenticated encryption and
///     turn AES128 into something that can encrypt an arbitrarily large amount of data.
/// - **SIV**: Galois counter mode requires a random value to encrypt. This scheme uses a Synthetic
///     Initialization Vector (SIV), which makes it so that the randomness is not very important for
///     security.
///
/// In total, this is a very safe-to-use encryption scheme that provided authenticated encryption
/// with additional data. A common alternative to this that provides the same guarantees (except for
/// nonce reuse resistance) is ChaCha20-Poly1305. Both are good options.
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedMessage {
    /// The actual ciphertext. This is what you usually think about when thinking about the
    /// encrypted message
    pub ciphertext: Vec<u8>,
    /// The nonce used to encrypt the ciphertext. This is necessary for decryption, but doesn't
    /// carry any data itself. The same nonce in theory shouldn't be used multiple times, but
    /// it's not super important with the SIV protocol which is nonce-reuse resistant.
    pub nonce: Vec<u8>,
    /// Some associated data. If this is present, it is necessary to successsfully decrypt the
    /// message
    pub associated_data: Option<Vec<u8>>,
}

/// We implement debug ourselves so that we get hex debug out, which is a bit nicer to read. We
/// also interpret the nonce as utf8, as all tests require that
impl std::fmt::Debug for EncryptedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedMessage")
            .field("ciphertext", &self.ciphertext.encode_hex::<String>())
            .field("nonce", &String::from_utf8(self.nonce.clone()))
            .field(
                "associated_data",
                &self
                    .associated_data
                    .clone()
                    .map(|aad| aad.encode_hex::<String>()),
            )
            .finish()
    }
}

/// Encrypt the BASE_MESSAGE with AES128-GCM-SIV and the nonce in the argument.
///
/// This should use the key SYM_KEY. Using GenericArray::from_slice, you can get a generic array
/// out of an &[u8].
pub fn encrypt_message_no_aad(msg: &[u8], nonce: &[u8]) -> EncryptedMessage {
    EncryptedMessage {
        ciphertext: match Aes128GcmSiv::new(GenericArray::from_slice(SYM_KEY))
            .encrypt(Nonce::from_slice(nonce), msg)
        {
            Ok(ciphertext) => ciphertext,
            Err(error) => panic!("encrypt_message_no_aad failed with {}.", error),
        },
        nonce: nonce.to_vec(),
        associated_data: None,
    }
}

/// Decrypt a message encrypted with AES128-GCM-SIV.
///
/// This should use the key SYM_KEY.
pub fn decrypt_message_no_aad(msg: EncryptedMessage) -> Result<Vec<u8>, ()> {
    match Aes128GcmSiv::new(GenericArray::from_slice(SYM_KEY)).decrypt(
        Nonce::from_slice(msg.nonce.as_slice()),
        Payload {
            msg: msg.ciphertext.as_slice(),
            aad: b"",
        },
    ) {
        Ok(decrypted) => Ok(decrypted),
        Err(_) => Err(()),
    }
}

/// Encrypt the BASE_MESSAGE with AES128-GCM-SIV and the nonce in the argument, also including some
/// additional authenticated data (AAD).
///
/// This should use the key SYM_KEY.
pub fn encrypt_message_with_aad(msg: &[u8], nonce: &[u8], aad: &[u8]) -> EncryptedMessage {
    EncryptedMessage {
        ciphertext: match Aes128GcmSiv::new(GenericArray::from_slice(SYM_KEY))
            .encrypt(Nonce::from_slice(nonce), Payload { msg: msg, aad: aad })
        {
            Ok(ciphertext) => ciphertext,
            Err(error) => panic!("encrypt_message_with_aad failed with {}", error),
        },
        nonce: nonce.to_vec(),
        associated_data: Some(aad.to_vec()),
    }
}

/// Decrypt a message encrypted with AES128-GCM-SIV. The message may have AAD.
///
/// This should use the key SYM_KEY.
pub fn decrypt_message(msg: EncryptedMessage) -> Result<Vec<u8>, ()> {
    match Aes128GcmSiv::new(GenericArray::from_slice(SYM_KEY)).decrypt(
        Nonce::from_slice(msg.nonce.as_slice()),
        Payload {
            msg: msg.ciphertext.as_slice(),
            aad: msg.associated_data.as_deref().unwrap_or(b""),
        },
    ) {
        Ok(decrypted) => Ok(decrypted),
        Err(_) => Err(()),
    }
}

use rand::{rngs::SmallRng, SeedableRng};
use rand_core::OsRng;
use sp_core::ed25519::{Pair as Ed25519Pair, Public as Ed25519Public};
use sp_core::{blake2_128, Pair};
use x25519_dalek_ng::{EphemeralSecret, PublicKey as X25519PublicKey, SharedSecret, StaticSecret};

// The following two conversion functions were necessary in order to make these two cryptography
// libraries compatible (sp_core and x25519_dalek). They both use the same underlying
// cryptography, so you wouldn't expect it to be so difficult. However, these two functions
// required a careful reading of all the source code, and go way, way below the abstraction
// layer. In general, don't do this unless you absolutely have to _and_ know what you're doing.
// This was done so that there is a way to integrate all of these primitives into one cohesive
// problem, because unfortunately the sp_core cryptography libraries do not support any type of
// shared secret from asymmetric cryptography.

///////////////////////////////////////////////////////////////////////////
/////////////////////// CRYPTO DANGER ZONE ////////////////////////////////
///////////////////////////////////////////////////////////////////////////

/// Convert an sp_core ed25519 public key into one compatible with x25519
pub fn sp_core_ed25519_public_to_x25519_public(public: Ed25519Public) -> X25519PublicKey {
    use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};

    let public_bytes = public.0;
    let decompressed = CompressedEdwardsY(public_bytes).decompress().unwrap();
    let montgomery = decompressed.to_montgomery();
    montgomery.0.into()
}

/// Convert an sp_core ed25519 pair into a X25519 static secret, which is the comparable struct
/// in the X25519 library.
pub fn sp_core_ed25519_pair_to_x25519_static_secret(pair: Ed25519Pair) -> StaticSecret {
    use sha2::{Digest, Sha512};

    let seed_bytes = pair.seed();
    let hashed = sha2::Sha512::digest(&seed_bytes);
    let mut truncated = [0u8; 32];
    truncated[..].copy_from_slice(&hashed.as_slice()[0..32]);
    StaticSecret::from(<[u8; 32]>::try_from(truncated).unwrap())
}

/// This is just for convenience, as writing `let x: X25519PublicKey = (&secret).into()` is just
/// awkward
pub fn x25519_public(secret: &StaticSecret) -> X25519PublicKey {
    secret.into()
}

///////////////////////////////////////////////////////////////////////////
///////////////////////// END DANGER ZONE /////////////////////////////////
///////////////////////////////////////////////////////////////////////////

/// Calculate a shared secret using diffie-hellman.
///
/// This is implemented by the x25519_dalek_ng crate imported above, which is also what the
/// methods above are there to help with.
pub fn calculate_shared_secret(my_pair: Ed25519Pair, their_public: Ed25519Public) -> SharedSecret {
    sp_core_ed25519_pair_to_x25519_static_secret(my_pair)
        .diffie_hellman(&sp_core_ed25519_public_to_x25519_public(their_public))
}

/// In the next few parts, we will need to generate ephemeral asymmetric key pairs. We will use
/// this as the RNG seed in order to keep the code easily testable. Specifically, we will always
/// use `SmallRng::seed_from_u64(RNG_SEED)` to create our RNGs. Cryptographic RNG is not
/// necessary here in the test code. Do not seed from a u64 when it matters.
const RNG_SEED: u64 = 2023;

/// We will be implementing an extremely simple form of hybrid cryptography. The scheme goes
/// like this:
///
/// ### Sending
///
/// 1. The sender finds the recipient's public key.
/// 1. The sender gets a shared secret with their own pair and the recipients public key.
/// 1. The sender uses the lower 16 bytes of the shared secret as the secret key for an
///     AES128-GCM-SIV cipher.
/// 1. The sender hashes the plaintext message using blake2_128 and takes the lower 12 bytes of
///     that as the nonce.
/// 1. The sender encrypts the message using the secret key and nonce, and sends the ciphertext
///     along with their own public key.
///
/// ### Receiving
///
/// 1. The receiver gets the shared secret with their own pair and the senders public key.
/// 1. The receiver uses the lower 16 bytes of the shared secret as the secret key for an
///     AES128-GCM-SIV cipher.
/// 1. The receiver uses the nonce and secret key to decrypt the message.
#[derive(Clone, PartialEq, Eq)]
pub struct HybridCryptographyMessage {
    pub ciphertext: Vec<u8>,
    /// The public key of the sender
    pub public: Ed25519Public,
    pub nonce: Vec<u8>,
}

/// We implement debug ourselves so that we get hex debug out, which is a bit nicer to read
impl std::fmt::Debug for HybridCryptographyMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridCryptographyMessage")
            .field("ciphertext", &self.ciphertext.encode_hex::<String>())
            .field("nonce", &self.nonce.encode_hex::<String>())
            .field("public", &self.public)
            .finish()
    }
}

/// Encrypt with hybrid scheme described over HybridCryptographyMessage
pub fn encrypt_hybrid(
    my_pair: Ed25519Pair,
    msg: &[u8],
    recipient: Ed25519Public,
) -> HybridCryptographyMessage {
    let shared_secret = calculate_shared_secret(my_pair, recipient);
    let nonce = blake2_128(msg).as_slice()[..12].to_vec();
    HybridCryptographyMessage {
        ciphertext: Aes128GcmSiv::new(GenericArray::from_slice(&shared_secret.as_bytes()[..16]))
            .encrypt(Nonce::from_slice(&nonce), msg)
            .unwrap(),
        public: my_pair.public(),
        nonce,
    }
}

/// Decrypt with hybrid scheme described over HybridCryptographyMessage
pub fn decrypt_hybrid(my_pair: Ed25519Pair, msg: HybridCryptographyMessage) -> Result<Vec<u8>, ()> {
    let shared_secret = calculate_shared_secret(my_pair, msg.public);
    match Aes128GcmSiv::new(GenericArray::from_slice(&shared_secret.as_bytes()[..16])).decrypt(
        Nonce::from_slice(msg.nonce.as_slice()),
        Payload {
            msg: msg.ciphertext.as_slice(),
            aad: b"",
        },
    ) {
        Ok(decrypted) => Ok(decrypted),
        Err(_) => Err(()),
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
    3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_message_test() {
        // Must be 96 bits -> 12 bytes -> 24 hex chars
        let nonce = hex::decode("beefbeefbeefbeefbeefbeef").unwrap();
        assert_eq!(
        EncryptedMessage {
            associated_data: None,
            ciphertext: hex::decode("190606237a4e03658399bfdf2ba9a60c87b150d450566ca16dca98de3757993b14ad4e2c7f99faba18c6041247d0bb2fb66d096f95c715e9ef").unwrap(),
            nonce: nonce.to_vec()
        },
        encrypt_message_no_aad(BASE_MESSAGE, &nonce)
    );
    }

    #[test]
    fn decrypt_message_no_aad_test() {
        let msg = EncryptedMessage {
        associated_data: None,
        ciphertext: hex::decode("190606237a4e03658399bfdf2ba9a60c87b150d450566ca16dca98de3757993b14ad4e2c7f99faba18c6041247d0bb2fb66d096f95c715e9ef").unwrap(),
        nonce: hex::decode("beefbeefbeefbeefbeefbeef").unwrap(),
    };

        assert_eq!(
            decrypt_message_no_aad(msg.clone()),
            Ok(BASE_MESSAGE.to_vec())
        );

        let mut bad_msg = msg.clone();
        // Corrupt the message, or otherwise change it
        for i in 15..18 {
            bad_msg.ciphertext[i] = i as u8;
        }
        // It's hard to show this by doing, but it is impossible for someone who doesn't know the
        // secret to modify this because AES128-GCM-SIV is a form of authenticated encryption.
        assert_eq!(decrypt_message_no_aad(bad_msg), Err(()));
    }

    #[test]
    fn encrypt_with_aad_test() {
        let nonce = hex::decode("beefbeefbeefbeefbeefbeef").unwrap();
        let aad = b"for pba use only";
        assert_eq!(
        EncryptedMessage {
            associated_data: Some(aad.to_vec()),
            ciphertext: hex::decode("858d6d71f4405652eefe0a162db81a9232bca717b03645e904c226632559d1e65a98b413f2e23b234d2d224b037e1921fba580e51c8908b9fd").unwrap(),
            nonce: nonce.to_vec()
        },
        encrypt_message_with_aad(BASE_MESSAGE, &nonce, aad)
    );
    }

    #[test]
    fn decrypt_message_test() {
        let nonce = hex::decode("beefbeefbeefbeefbeefbeef").unwrap();
        let aad = b"for pba use only";
        let msg = EncryptedMessage {
        associated_data: Some(aad.to_vec()),
        ciphertext: hex::decode("858d6d71f4405652eefe0a162db81a9232bca717b03645e904c226632559d1e65a98b413f2e23b234d2d224b037e1921fba580e51c8908b9fd").unwrap(),
        nonce: nonce.to_vec()
    };
        let msg_with_wrong_aad = EncryptedMessage {
            associated_data: Some(b"a malicious purpose".to_vec()),
            ..msg.clone()
        };

        assert_eq!(Ok(BASE_MESSAGE.to_vec()), decrypt_message(msg.clone()));
        assert_eq!(Err(()), decrypt_message(msg_with_wrong_aad));

        let msg_encrypted_without_aad = EncryptedMessage {
        associated_data: None,
        ciphertext: hex::decode("190606237a4e03658399bfdf2ba9a60c87b150d450566ca16dca98de3757993b14ad4e2c7f99faba18c6041247d0bb2fb66d096f95c715e9ef").unwrap(),
        nonce: hex::decode("beefbeefbeefbeefbeefbeef").unwrap(),
    };

        assert_eq!(
            decrypt_message(msg_encrypted_without_aad),
            Ok(BASE_MESSAGE.to_vec())
        );
    }

    #[test]
    fn calculate_shared_secret_test() {
        let alice = Ed25519Pair::from_string("//Alice", None).unwrap();
        let bob = Ed25519Pair::from_string("//Bob", None).unwrap();

        let s_alice_bob = calculate_shared_secret(alice.clone(), bob.public());
        let s_bob_alice = calculate_shared_secret(bob.clone(), alice.public());

        assert_eq!(
            s_alice_bob.as_bytes(),
            hex::decode("ae7f8cdfab1f4776f1ba821e9555752057a1258915cbc827a4bc7b38156ce122")
                .unwrap()
                .as_slice()
        );
        assert_eq!(s_alice_bob.to_bytes(), s_bob_alice.to_bytes());
    }
}

#[cfg(test)]
mod optional_tests {
    use super::*;

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn encrypt_hybrid_test() {
        let alice = Ed25519Pair::from_string("//Alice", None).unwrap();
        let bob = Ed25519Pair::from_string("//Bob", None).unwrap();

        assert_eq!(
        HybridCryptographyMessage {
            ciphertext: hex::decode("19ca867857e2a81dfdb9e5bc061bb0bd6b7b41d659f0f645673125b7658987d2ff8080f75d1aea44c8ef4aac91cd97c5bf9f6eab5699a22a3f").unwrap(),
            public: alice.public(),
            nonce: hex::decode("02107f69bbe4c314f9dd1d96").unwrap(),
        },
        encrypt_hybrid(alice, BASE_MESSAGE, bob.public())
    )
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn decrypt_hybrid_test() {
        let alice = Ed25519Pair::from_string("//Alice", None).unwrap();
        let bob = Ed25519Pair::from_string("//Bob", None).unwrap();

        assert_eq!(
        Ok(BASE_MESSAGE.to_vec()),
        decrypt_hybrid(bob,
        HybridCryptographyMessage {
            ciphertext: hex::decode("19ca867857e2a81dfdb9e5bc061bb0bd6b7b41d659f0f645673125b7658987d2ff8080f75d1aea44c8ef4aac91cd97c5bf9f6eab5699a22a3f").unwrap(),
            public: alice.public(),
            nonce: hex::decode("02107f69bbe4c314f9dd1d96").unwrap(),
        })
    )
    }
}
