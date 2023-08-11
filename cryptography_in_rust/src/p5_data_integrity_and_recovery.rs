#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::{cell::RefCell, collections::VecDeque};

/// This struct represents a lossy channel that messages might be sent across. It has a very
/// small probability of changing every byte that gets passed in. Each byte will be changed with
/// probability equal to CORRUPTION_RATE / 100_000. Corruption rates are assumed to be in the
/// range [0, 100_000]. For simplicity, this channel only corrupts bytes, but does not actually
/// drop bytes.
///
/// However, sending things over a lossy channel makes it hard for applications to work on it! We
/// are going to implement two protocols that operate over a lossy channel to provide different
/// guarantees about the communication over the channel.
pub struct LossyChannel {
    corruption_rate: u32,
    total_bytes_sent: usize,
    message_queue: VecDeque<Vec<u8>>,
}

impl LossyChannel {
    pub fn new(corruption_rate: u32) -> Self {
        if corruption_rate > 100_000 {
            panic!("corruption rate too large")
        }
        LossyChannel {
            corruption_rate,
            total_bytes_sent: 0,
            message_queue: Default::default(),
        }
    }

    /// Send a message over the lossy channel
    pub fn send(&mut self, msg: &[u8]) {
        self.message_queue.push_back(msg.to_vec());
        self.total_bytes_sent += msg.len();
    }

    /// Receive a message from the lossy channel. It may be corrupted.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let msg = self.message_queue.pop_front();
        Some(
            msg?.into_iter()
                .map(|b| {
                    // This generates a random float in the range [0, 1)
                    if rand::random::<f32>() < (self.corruption_rate as f32) / 100_000.0 {
                        rand::random::<u8>()
                    } else {
                        b
                    }
                })
                .collect(),
        )
    }
}

/// A trait for senders over a channel
pub trait ChannelSender {
    fn send(&self, msg: &[u8]);
}

pub enum ReceivingError {
    ChannelEmpty,
    MessageCorrupted,
}

/// A trait for receivers from a channel
pub trait ChannelReceiver {
    fn receive(&self) -> Result<Vec<u8>, ReceivingError>;
}

use crate::p1_hashing::{add_integrity_check, verify_data_integrity};

pub struct HashIntegritySender<'a> {
    // self.channel.borrow_mut().send() lets you send a message through the channel
    channel: &'a RefCell<LossyChannel>,
}

pub struct HashIntegrityReceiver<'a> {
    // self.channel.borrow_mut().receive() lets you receive a message from the channel
    channel: &'a RefCell<LossyChannel>,
}

impl<'a> ChannelSender for HashIntegritySender<'a> {
    /// To send over the hash integrity channel, use the functions you wrote in the hashes
    /// module to add an integrity check before sending a message.
    ///
    /// This protocol provides an integrity guarantee to the message sent over the channel.
    /// Specifically, it says "If the message is corrupted, you will know."
    fn send(&self, msg: &[u8]) {
        let msg_with_integrity = add_integrity_check(msg);
        self.channel.borrow_mut().send(&msg_with_integrity);
    }
}

impl<'a> ChannelReceiver for HashIntegrityReceiver<'a> {
    /// To receive a message, check the integrity of the message before returning it. If the
    /// integrity check fails, return an error
    fn receive(&self) -> Result<Vec<u8>, ReceivingError> {
        match self.channel.borrow_mut().receive() {
            None => Err(ReceivingError::ChannelEmpty),
            Some(received_msg) => match verify_data_integrity(received_msg.clone()) {
                Ok(_) => Ok(received_msg),
                Err(_) => Err(ReceivingError::MessageCorrupted),
            },
        }
    }
}

use reed_solomon::Decoder as ECCDecoder;
use reed_solomon::Encoder as ECCEncoder;

pub struct ErrorCorrectingCodeSender<'a> {
    // self.channel.borrow_mut().send() lets you send a message through the channel
    channel: &'a RefCell<LossyChannel>,
    /// The number of corrupt bytes that can be corrected per (255 - 2 * correctable_errors) bytes
    /// of the message. This parameter must be the same on both the sender and receiver, and is
    /// one half of ecc_len. Behavior is undefined if correctable_errors > 127.
    pub correctable_errors: usize,
}

pub struct ErrorCorrectingCodeReceiver<'a> {
    // self.channel.borrow_mut().receive() lets you receive a message from the channel
    channel: &'a RefCell<LossyChannel>,
    /// The number of corrupt bytes that can be corrected per (255 - 2 * correctable_errors) bytes
    /// of the message. This parameter must be the same on both the sender and receiver, and is
    /// one half of ecc_len. Behavior is undefined if correctable_errors > 127.
    pub correctable_errors: usize,
}

impl<'a> ChannelSender for ErrorCorrectingCodeSender<'a> {
    /// Send a message using erasure codes to ensure that up to self.correctable_errors can be
    /// fixed.
    ///
    /// This protocol provides error correction and integrity to the message sent over the channel.
    /// Specifically, it says "If up to self.correctable_errors bytes are corrupted, we'll fix them
    /// for you. If more than that is corrupted, we'll tell you the message is corrupted."
    ///
    /// An implementation detail of the crate we are using for reed solomon codes that
    /// unfortunately leaks out is that the total size of the message + the error correcting
    /// bytes can be 255 bytes at most. Thus, if we receive a message that would be too
    /// large, we will break it up into segments that are exactly 255 bytes total with the
    /// error correction, and apply that level of error correction to each. A more
    /// production-level crate would not have this problem.
    ///
    /// You may find the `chunks` method on slices very useful for this problem.
    fn send(&self, msg: &[u8]) {
        let data_block_size = 255 - 2 * self.correctable_errors;
        let encoder = ECCEncoder::new(2 * self.correctable_errors);
        todo!("OPTIONAL");
    }
}

impl<'a> ChannelReceiver for ErrorCorrectingCodeReceiver<'a> {
    /// Receive a message that uses error correcting codes to ensure that up to
    /// self.correctable_errors can be fixed.
    ///
    /// This will also have to do block-level decoding and handling of error correcting codes.
    fn receive(&self) -> Result<Vec<u8>, ReceivingError> {
        todo!("OPTIONAL")
    }
}

////////////////////////////////////////////////////////////////////////////
////////////// Tests and test utils are below this line ////////////////////
////////////////////////////////////////////////////////////////////////////

/// This test is for you to explore how these things work as the parameters change. We will
/// compare sending messages with integrity checks, and with error correction codes. Make sure
/// to run the test with --nocapture, as it will never fail! Also, run it with `--ignored`,
/// otherwise the test will be ignored!
///
/// cargo test --package pba-assignment --lib -- p5_data_integrity_and_recovery::lossy_channel_exploratory_test --exact --nocapture --ignored
#[test]
#[ignore]
fn lossy_channel_exploratory_test() {
    use optional_tests::{run_test, ProtocolType, TestParams, TestReturns};
    // Parameters that control the overall test
    // Feel free to change these to see how the code reacts!
    let params = TestParams {
        message_count: 1000,
        message_length: 150,
        corruption_rate: 100,
        ecc_correctable_errors: 10,
    };

    let protocols: Vec<ProtocolType> = vec![
        ProtocolType::HashIntegrity,
        // Uncomment this once you complete the error correcting code protocol
        ProtocolType::ErrorCorrectingCode,
    ];

    let f_corr_rate = params.corruption_rate as f64 / 100_000.0;
    println!("\n\nTesting with:");
    println!("  {} messages", params.message_count);
    println!("  {} bytes each", params.message_length);
    println!("  {:.5} byte corruption rate", f_corr_rate);
    // You might notice that this expected uncorrupted messages consistently underestimates the
    // actual number of messages corrected with the hash integrity protocol. Can you figure out
    // why that might be?
    println!(
        "  Expected uncorrupted messages: {:.2}%",
        100.0 * (1.0 - f_corr_rate).powf(params.message_length as f64)
    );
    println!(
        "  Expected errors per message: {}",
        f_corr_rate * params.message_length as f64
    );

    for protocol in protocols {
        let ret = run_test(params.clone(), protocol.clone());

        println!("\nSummary for {:?}:", protocol);
        println!(
            "  Total data bytes: {}",
            params.message_count * params.message_length
        );
        println!(
            "  Amount transmitted: {} ({}% extra)",
            ret.sent_bytes,
            100.0 * (ret.sent_bytes - params.message_count * params.message_length) as f32
                / (params.message_count * params.message_length) as f32
        );
        println!(
            "  Messages successful: {} ({:.2}%)",
            ret.msgs_successful,
            100.0 * ret.msgs_successful as f32 / params.message_count as f32
        );
        println!(
            "  Messages corrupted: {} ({:.2}%)",
            ret.msgs_corrupted,
            100.0 * ret.msgs_corrupted as f32 / params.message_count as f32
        );
        if ret.bad_messages_let_through != 0 {
            println!(
                "  Bad messages let through: {}",
                ret.bad_messages_let_through
            );
        }
    }
}

/// This function is not graded. It is just for collecting feedback.
/// On a scale from 0 - 100, with zero being extremely easy and 100 being extremely hard, how hard
/// did you find the exercises in this section?
pub fn how_hard_was_this_section() -> u8 {
    todo!()
}

/// This function is not graded. It is just for collecting feedback.
/// About how much time (in hours) did you spend on the exercises in this section?
pub fn how_many_hours_did_you_spend_on_this_section() -> f32 {
    todo!()
}

#[cfg(test)]
mod optional_tests {
    use super::*;

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn hash_integrity_works_with_no_corruption() {
        let params = TestParams {
            message_count: 1000,
            message_length: 150,
            corruption_rate: 0,
            ecc_correctable_errors: 1,
        };

        let hi_ret = run_test(params.clone(), ProtocolType::HashIntegrity);
        assert_eq!(1000, hi_ret.msgs_successful);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn error_correction_works_with_no_corruption() {
        let params = TestParams {
            message_count: 1000,
            message_length: 150,
            corruption_rate: 0,
            ecc_correctable_errors: 1,
        };

        let ecc_ret = run_test(params.clone(), ProtocolType::ErrorCorrectingCode);
        assert_eq!(1000, ecc_ret.msgs_successful);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn hash_detects_and_rejects_corruption() {
        let params = TestParams {
            message_count: 1000,
            message_length: 150,
            corruption_rate: 100,
            ecc_correctable_errors: 10,
        };

        let hi_ret = run_test(params.clone(), ProtocolType::HashIntegrity);
        // We put very loose bounds in the test, because it is randomized
        assert!(hi_ret.msgs_successful > 700);
        assert!(hi_ret.msgs_corrupted > 75);
        assert_eq!(0, hi_ret.bad_messages_let_through);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn ecc_corrects_corruption() {
        let params = TestParams {
            message_count: 1000,
            message_length: 150,
            corruption_rate: 100,
            ecc_correctable_errors: 10,
        };

        let ecc_ret = run_test(params.clone(), ProtocolType::ErrorCorrectingCode);
        // We put very loose bounds in the test, because it is randomized
        assert!(ecc_ret.msgs_successful > 950);
        assert_eq!(0, ecc_ret.bad_messages_let_through);
    }

    #[test]
    #[cfg_attr(not(feature = "optional-tests"), ignore)]
    fn hash_detects_and_rejects_heavy_corruption() {
        let params = TestParams {
            message_count: 1000,
            message_length: 150,
            // Almost all bytes are garbage now
            corruption_rate: 80_000,
            ecc_correctable_errors: 4,
        };

        // Do you think this works for error correcting codes? Give it a try!
        let hi_ret = run_test(params.clone(), ProtocolType::HashIntegrity);
        assert_eq!(1000, hi_ret.msgs_corrupted);
        assert_eq!(0, hi_ret.bad_messages_let_through);
    }

    #[derive(Debug, Clone)]
    pub enum ProtocolType {
        HashIntegrity,
        ErrorCorrectingCode,
    }

    /// Parameters for a test of protocols over a lossy channel
    #[derive(Clone)]
    pub struct TestParams {
        /// The length of each message sent in the test, in bytes
        pub message_length: usize,
        /// The number of messages to send
        pub message_count: usize,
        /// The number of bytes per 100,000 that get corrupted
        pub corruption_rate: u32,
        /// The number of errors capable of being corrected using ECCs
        /// This is ignored if not using ECCs
        pub ecc_correctable_errors: usize,
    }

    /// Return values for the randomized test of protocols over a lossy channel
    pub struct TestReturns {
        /// The number of data bytes in all messages
        pub data_bytes: usize,
        /// The total number of bytes sent over the channel
        pub sent_bytes: usize,
        /// The number of successful messages
        pub msgs_successful: usize,
        /// The number of corrupted messages
        pub msgs_corrupted: usize,
        /// The number of bad messages let through
        pub bad_messages_let_through: usize,
    }

    /// Run a randomized test on a lossy channel with the given parameters
    ///
    /// This code does not need to be changed, but you can read it if you're curious.
    pub fn run_test(params: TestParams, protocol: ProtocolType) -> TestReturns {
        use rand::{thread_rng, Rng};

        let msgs: Vec<Vec<u8>> = (0..params.message_count)
            .map(|_| {
                let mut msg = vec![0u8; params.message_length];
                thread_rng().fill(&mut msg[..]);
                msg
            })
            .collect();

        let channel = LossyChannel::new(params.corruption_rate);
        let rc_channel = RefCell::new(channel);
        let mut msgs_successful = 0;
        let mut msgs_corrupted = 0;
        let mut msgs_corrupted_let_through = 0;
        let sender: Box<dyn ChannelSender> = match protocol {
            ProtocolType::HashIntegrity => Box::new(HashIntegritySender {
                channel: &rc_channel,
            }),
            ProtocolType::ErrorCorrectingCode => Box::new(ErrorCorrectingCodeSender {
                channel: &rc_channel,
                correctable_errors: params.ecc_correctable_errors,
            }),
        };
        let receiver: Box<dyn ChannelReceiver> = match protocol {
            ProtocolType::HashIntegrity => Box::new(HashIntegrityReceiver {
                channel: &rc_channel,
            }),
            ProtocolType::ErrorCorrectingCode => Box::new(ErrorCorrectingCodeReceiver {
                channel: &rc_channel,
                correctable_errors: params.ecc_correctable_errors,
            }),
        };

        msgs.iter().for_each(|msg| {
            sender.send(msg);
            match receiver.receive() {
                Ok(rec_msg) => {
                    if &rec_msg == msg {
                        msgs_successful += 1;
                    } else {
                        msgs_corrupted_let_through += 1;
                    }
                }
                Err(_) => msgs_corrupted += 1,
            }
        });

        let sent_bytes = rc_channel.borrow().total_bytes_sent;

        return TestReturns {
            data_bytes: params.message_count * params.message_length,
            sent_bytes,
            msgs_successful,
            msgs_corrupted,
            bad_messages_let_through: msgs_corrupted_let_through,
        };
    }
}
