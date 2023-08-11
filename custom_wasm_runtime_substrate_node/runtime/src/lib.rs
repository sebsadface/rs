//! Welcome to the `Frame-less` exercise, the third edition.
//!
//! > This assignment is based on Joshy's experiment, years ago, to explore building a Substrate
//! > runtime using pure Rust. If you learn something new in this exercise, attribute it to his
//! > work. We hope you to also explore new possibilities, and share it with other for education.
//!
//! > This assignment builds on top of the `mini_substrate` section of the pre-course material. It
//! > is highly recommended to re-familiarize yourself with that first.
//!
//! ## Context
//!
//! As the name suggest, this is Frame-less runtime. It is a substrate-compatible runtime, which you
//! can easily run with companion `node`, without using `frame`.
//!
//! To run the `node`, execute `cargo run -- --dev`, possibly with `--release`. `--dev` will ensure
//! that a new database is created each time, and your chain starts afresh.
//!
//! While you are welcome to explore the `node` folder, it is not part of this assignment, and you
//! can leave it as-is.
//!
//! This node uses a testing block-authoring/consensus scheme in which a block is produced at fixed
//! intervals. See `--consensus` cli option.
//!
//! ## Warm Up
//!
//! First, study the runtime code with the help of your instructor. You will soon realize that it is
//! missing some key components. Most notably, the logic to:
//!
//! 1. apply extrinsics
//! 2. validate extrinsics for the tx-pool
//! 3. finalize a block upon authoring
//!
//! are not complete. Your first task is to complete them, and make sure all local tests are
//! passing. provide a simple `apply_extrinsic`, and finish `finalize_block`. For this first
//! section, you can leave the tx-pool api as-is.
//!
//! * For `apply_extrinsic`, start by only implementing [`shared::SystemCall::Set`] for now, this is
//!   the only one used in tests.
//! * For `finalize_block`, your main task is to obtain the `raw_header` that is placed onchain in
//!   `initialize_block`, and to place a correct `state_root` and `extrinsics_root` in it.
//!
//! Fixing `finalize_block` makes the block be a valid import-able block. This test demonstrates
//! this property:
#![doc = docify::embed!("src/lib.rs", import_and_author_equal)]
//!
//! Also, if you run your chain with two nodes, you will be able to test this property. See Hints
//! section below.
//!
//! You will soon realize that you are asked to implement proper signature checking. In this
//! assignment, we are using the types from [`sp_runtime::generic`] (see [`shared`]). Studying these
//! types and how the handle signatures within themselves should help you implement proper signature
//! checking.
//!
//! All in all, your runtime should only work with signed extrinsics, and instantly reject unsigned
//! (or badly signed) extrinsics. See the following tests:
#![doc = docify::embed!("src/lib.rs", bad_signature_fails)]
//!
#![doc = docify::embed!("src/lib.rs", unsigned_set_value_does_not_work)]
//!
//! By the end of this section, you should fix the aforementioned 3 parts of your runtime, implement
//! proper dispatch logic for [`shared::SystemCall`], and this should enable you yo pass all tests
//! in this file. Moreover, your should be able to play with your blockchain, through running a
//! node, and interacting with it via `curl`, `wscat` or a similar tool. See `encode_examples` test.
//!
//! > Most of [`shared::SystemCall`] instances are for you to use for learning. Try and upgrade your
//! > chain using [`shared::SystemCall::Upgrade`]!
//!
//! ## Main Task
//!
//! Once you are done with the above, you can start your main objective, which is to re-implement
//! everything you have done for `mini_substrate` here. That is, implement a simple currency system,
//! with abilities to mint, transfer and reserve, and a staking system on top of it. Additionally,
//! we are asking you to add a basic tip and nonce system as well.
//!
//! ## Specification
//!
//! ### Dispatchables
//!
//! Similar to `mini_substrate`, the exact expectation of behavior for each instance of
//! [`shared::RuntimeCallExt`] is document in its own documentation. This should be identical to
//! what you had to code for `mini_substrate`.
//!
//! > As noted above, whether you want to use a trait like `Dispatchable` or not is up to you.
//!
//! ### Storage
//!
//! Exact same expectation as `mini_substrate`.
//!
//! * mapping [`shared::AccountId`] to [`shared::AccountBalance`] kept at `BalancesMap +
//!   encode(account)`.
//! * value of type [`shared::Balance`] for total issuance kept at `TotalIssuance`.
//!
//! > Again, you are free to use the same storage abstractions as in `mini_substrate` or not. We
//! > highly advice you do ;)
//!
//! ### Additional Logic
//!
//! Notable new desired behavior, compared to `mini_substrate`:
//!
//! #### 1. Tipping
//!
//! The final [`shared::Extrinsic`]'s `Call`, namely [`shared::RuntimeCallExt`] contains `tip`
//! field. As the name suggests, this is some additional funds that are sent by the user to chain.
//! Other than this optional tip, all other extrinsics are free.
//!
//! Tipped extrinsics are prioritized over non-tipped ones through the virtue of a higher
//! `priority`. This is further explained in `validate_transaction` section below.
//!
//! Deducting the tip from the sender must happen prior to executing anything else in the extrinsic.
//! Failure to pay for fees is always a full failure of the extrinsic (similar to a failed signature
//! check).
//!
//! Once the tip is deducted, it is added to an account id specified by [`shared::TREASURY`]. The
//! same rules about account creation apply to this account as well. If the tip is not enough to
//! create this account, then the execution must succeed, and the amount of tip must be burnt (the
//! sender does not get it back, but neither the recipient gets it).
//!
//! Total issuance must be kept up to date in all of the cases above.
//!
//! #### 2. Account Creation/Deletion
//!
//! An account that starts a transaction with non-zero `free` and `reserved`, but finishes it with
//! equal to zero values for `free` and `reserved` (by `TransferAll` or equivalent) is notionally
//! "destroyed". Such an account should not be kept in storage with a value like
//! `Default::default()`. Instead, it should be removed from storage. This is crucial to save space
//! in your blockchain.
//!
//! In such cases, the nonce information is also forgotten. This is not how things work in a real
//! blockchain, as it opens the door for replay attacks, but we keep it like this for simplicity.
//!
//! An account that has no balance and only a non-zero `nonce` field can remain in storage. But, as
//! noted above, once the free balance is non-zero, when it collapses back to zero, the account must
//! be removed from storage.
//!
//! > As noted, the above scenario is somewhat unsound given the fee-less nature of the system.
//!
//!
//! #### Nonce
//!
//! You should implement a nonce system, as explained as a part of the tx-pool lecture. In short,
//! the validation of each transaction should `require` nonce `(sender, n-1)` and provide `(sender,
//! n)`. See `TaggedTransactionQueue` below for more information.
//!
//! Note that your nonce should be checked as a part of transaction pool api, which means it should
//! be implemented as efficiently as possibly, next to other checks that need to happen.
//!
//! ### `BlockBuilder::apply_extrinsic`
//!
//! One of your objectives is to implement the logic for `apply_extrinsic`. Here, we describe what
//! return value we expect from it.
//!
//! Recall that [`ApplyExtrinsicResult`] is essentially a nested `Result`. The outer one says
//! whether _applying_ the extrinsic to the block was fine, and the inner one says whether the
//! extrinsic itself was *dispatched* fine.
//!
//! For example, a failed transfer will still reside in a block, and is *applied* successfully, but
//! it is not *dispatched* successfully. So such an extrinsic should something like `Ok(Err(_))` in
//! its `apply_extrinsic`.
//!
//! Your `apply_extrinsic` should:
//!
//! * Return `Err` with [`sp_runtime::transaction_validity::TransactionValidityError::Invalid`] and
//!   [`sp_runtime::transaction_validity::InvalidTransaction::BadProof`] if the extrinsic has an
//!   invalid signature.
//! * Return `Err` with [`sp_runtime::transaction_validity::TransactionValidityError::Invalid`] and
//!   [`sp_runtime::transaction_validity::InvalidTransaction::Payment`] if the extrinsic cannot pay
//!   for its declared tip.
//! * Return `Err` with [`sp_runtime::transaction_validity::InvalidTransaction::Future`] or `Stale`
//!   if the nonce is too high or too low.
//!
//! In all other cases, outer `Result` is `Ok`.
//!
//! * If the inner dispatch is failing, your return value should look like `Ok(Err(_))`, and we
//!   don't care which variant of `DispatchError` you return.
//!
//! ### `TaggedTransactionQueue::validate_transaction`
//!
//! Recall that the return type of `validate_transaction` is
//! [`sp_runtime::transaction_validity::TransactionValidity`] which is simply a `Result`. Similar to
//! the above, your `validate_transaction` implementation must:
//!
//! * Return `Err` with [`sp_runtime::transaction_validity::TransactionValidityError::Invalid`] and
//!   [`sp_runtime::transaction_validity::InvalidTransaction::BadProof`] if the extrinsic has an
//!   invalid signature.
//! * Return `Err` with [`sp_runtime::transaction_validity::TransactionValidityError::Invalid`] and
//!   [`sp_runtime::transaction_validity::InvalidTransaction::Payment`] if the extrinsic cannot pay
//!   for its declared tip.
//! * Return `Err` with [`sp_runtime::transaction_validity::InvalidTransaction::Future`] or `Stale`
//!   if the nonce is too high or too low.
//!
//! Moreover, if tip is provided (and can paid at the time), the
//! [`sp_runtime::transaction_validity::ValidTransaction::priority`] must be set to the tip value
//! (use a simple saturated conversion if needed).
//!
//! ### `Core_execute_block`
//!
//! The `execute_block` expects a valid block in which all transactions will get included. That is,
//! it will expect all `ApplyExtrinsicResult` to be `Ok(_)`. Note that a failed dispatch is
//! acceptable, like `Ok(Err(_))`.
//!
//! You should not need to change this API, but studying it will be fruitful.
//!
//! ## Grading
//!
//! This assignment is primarily graded through automatic tests, not by looking at the internals of
//! your runtime. Manual grading is a small part. This means you should be very careful about
//! adhering to the rules and specifications.
//!
//! Automatic Wasm grading means:
//!
//! * we do not care about the internals of your runtime, other than the standard set of runtime
//!   apis.
//! * we do not care if you derive some additional trait for some type anywhere.
//! * but we do care about your storage layout being exactly as described in `mini_substrate`.
//! * we do care about the extrinsic format being exactly as described in [`shared`].
//! * our tests are fairly similar to `import_and_author_equal`. We construct a block, author it,
//!   then import it, then assert that the process was correct, and finally add a number of
//!   assertions about the state ourselves.
//!
//! While we can't force you not to change [`shared`] module, we use an exact copy of this file to
//! craft extrinsics/blocks to interact with your runtime, and we expect to find the types mentioned
//! in there (eg. [`shared::AccountBalance`]) to be what we decode from your storage.
//!
//! That being said, you can use types that are equivalent to their encoding to the ones mentioned
//! in [`shared`].
//!
//! Our tests are consisted of 5 main modules:
//!
//! * basics: this set of tests only check that you can properly execute the [`shared::SystemCall`]
//!   variants.
//! * `block_builder` and `validate_transaction` apis: these tests check the return type of these
//!   two runtime apis, excluding the details for tipping and nonce. They will use
//!   [`shared::SystemCall`].
//!
//! If you implement the above two correctly, you will be granted a score of 1.
//!
//! * correct implementation of the currency and staking system.
//!
//! If you implement the above correctly, you will be granted a score of 2.
//!
//! * correct implementation of the tipping and nonce system.
//!
//! If you implement the above correctly, you will be granted a score of 3 or more.
//!
//! We expect all students to pass the pillars marked above in a linear fashion. In other words, we
//! see it unlikely for you to be able to implement the currency system correctly, without
//! implementing the basics first. Students who don't follow this pattern will be assessed on a
//! case-by-case basis.
//!
//! > Given that this is the first time that this assignment is being auto-graded, you can request a
//! > pre-submit from your each of your teachers at most once. This is subject to availability.
//!
//! ## Hints
//!
//! ### Block Authoring vs. Import
//!
//! Recall that the api call flow in block authoring is:
//!
//! ```ignore
//! Core::initialize_block(raw_header);
//! loop {
//!     BlockBuilder::apply_extrinsic
//! }
//! BlockBuilder::finalize_block() -> final_header
//! ```
//!
//! The client builds a raw header that has `number` and a few other fields set, but no roots yet,
//! and passes it to the runtime in `initialize_block`. The runtime stored this raw header at this
//! point, and intends to return its final version in `finalize_block`.
//!
//! When importing a block, the api call flow is:
//!
//! ```ignore
//! Core::execute_block(block);
//! ```
//!
//! End of the day, you must ensure that the above two code paths come to the same state root, and
//! record it in the block header, along with the root of all extrinsics.
//!
//! ### Logging
//!
//! Logging can be enabled by setting the `RUST_LOG` environment variable, as such:
//!
//! ```ignore
//! RUST_LOG=frameless=debug cargo run
//! ```
//!
//! Or equally:
//!
//! ```ignore
//! cargo run -- --dev -l frameless=debug
//! ```
//!
//! ### Running Two Nodes
//!
//! In order to run two nodes, execute the following commands in two different terminals.
//!
//! ```ignore
//! cargo run -- --dev --alice -l frameless=debug
//! cargo r -- --dev --bob -l frameless=debug --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/<node-id-of-alice>
//! ```
//!
//! If you let the former `--alice` node progress for a bit, you will see that `--bob` will start
//! syncing from alice.
//!
//! ### EXTRA/OPTIONAL: `SignedExtensions`
//!
//! What we have implemented in this extra as added fields to our [`shared::RuntimeCallExt`] should
//! have ideally been implemented as a signed extension. In a separate branch, explore this, and ask
//! for our feedback. If make progress on this front, DO NOT submit it for grading, as our grading
//! will work with the simpler `RuntimeCallExt` model.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

const LOG_TARGET: &'static str = "frameless";

pub mod shared;

use log::info;
use parity_scale_codec::{Decode, Encode};
use shared::Block;

use sp_api::impl_runtime_apis;
use sp_runtime::{
	create_runtime_str,
	generic::{self},
	traits::{BlakeTwo256, Block as BlockT},
	transaction_validity::{
		self, InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
	},
	ApplyExtrinsicResult, DispatchError,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::{hexdisplay::HexDisplay, OpaqueMetadata, H256};
use sp_runtime::traits::Hash;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

use sp_version::RuntimeVersion;

use crate::shared::{
	AccountBalance, CurrencyCall, RuntimeCall, StakingCall, SystemCall, EXTRINSICS_KEY, HEADER_KEY,
	VALUE_KEY,
};

/// Opaque types. This is what the lectures referred to as `ClientBlock`. Notice how
/// `OpaqueExtrinsic` is merely a `Vec<u8>`.
pub mod opaque {
	use super::*;
	type OpaqueExtrinsic = sp_runtime::OpaqueExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<shared::BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, OpaqueExtrinsic>;
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("frameless-runtime"),
	impl_name: create_runtime_str!("frameless-runtime"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// The type that provides the genesis storage values for a new chain.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize, Default))]
pub struct RuntimeGenesisConfig;

#[cfg(feature = "std")]
impl BuildStorage for RuntimeGenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		// make sure to not remove this, as it might break the node code.
		storage.top.insert(well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec());

		// if you want more data in your genesis, add it here.
		Ok(())
	}
}

/// The main struct in this module. In frame this comes from `construct_runtime!` macro.
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
pub struct Runtime;

impl Runtime {
	fn print_state() {
		let mut key = vec![];
		while let Some(next) = sp_io::storage::next_key(&key) {
			let val = sp_io::storage::get(&next).unwrap().to_vec();
			log::trace!(
				target: LOG_TARGET,
				"{} <=> {}",
				HexDisplay::from(&next),
				HexDisplay::from(&val)
			);
			key = next;
		}
	}

	fn get_state<T: Decode>(key: &[u8]) -> Option<T> {
		sp_io::storage::get(key).and_then(|d| T::decode(&mut &*d).ok())
	}

	fn mutate_state<T: Decode + Encode + Default>(key: &[u8], update: impl FnOnce(&mut T)) {
		let mut value = Self::get_state(key).unwrap_or_default();
		update(&mut value);
		sp_io::storage::set(key, &value.encode());
	}

	fn do_initialize_block(header: &<Block as BlockT>::Header) {
		info!(
			target: LOG_TARGET,
			"Entering initialize_block. header: {:?} / version: {:?}", header, VERSION.spec_version
		);
		sp_io::storage::set(&HEADER_KEY, &header.encode());
		sp_io::storage::clear(&EXTRINSICS_KEY);
	}

	fn do_finalize_block() -> <Block as BlockT>::Header {
		info!(target: LOG_TARGET, "Entering finalize_block."); // Seb's Code
													   // fetch the header that was given to us at the beginning of the block.
		let mut header = Self::get_state::<<Block as BlockT>::Header>(HEADER_KEY)
			.expect("We initialized with header, it never got mutated, qed"); // Starter Code, but changed to `mut` by seb
																  // and make sure to _remove_ it.
		sp_io::storage::clear(&HEADER_KEY); // Starter Code

		// TODO: set correct state root and extrinsics root, as described in the corresponding test
		// case.

		// Set extrinsic root
		let extrinsics = Self::get_state::<Vec<Vec<u8>>>(EXTRINSICS_KEY).unwrap_or_default();
		header.extrinsics_root =
			BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

		// Set state root
		let raw_state_root = &sp_io::storage::root(VERSION.state_version())[..];
		header.state_root = H256::decode(&mut &raw_state_root[..]).unwrap();

		info!(target: LOG_TARGET, "Finishing block Finalize.");
		Self::print_state();

		header
	}

	/// Your code path to execute a block that has been previously authored.
	///
	/// Study this carefully, but you probably don't need to change it, other than providing a
	/// proper `do_apply_extrinsic`.
	fn do_execute_block(block: Block) {
		info!(target: LOG_TARGET, "Entering execute_block. block: {:?}", block);
		sp_io::storage::clear(&EXTRINSICS_KEY);

		for extrinsic in block.clone().extrinsics {
			let _outcome = Runtime::do_apply_extrinsic(extrinsic)
				.expect("A block author has provided us with an invalid block; bailing; qed");
		}

		// check state root. Clean the state prior to asking for the root.
		sp_io::storage::clear(&HEADER_KEY);

		// NOTE: if we forget to do this, how can you mess with the blockchain?
		let raw_state_root = &sp_io::storage::root(VERSION.state_version())[..];
		let state_root = H256::decode(&mut &raw_state_root[..]).unwrap();
		assert_eq!(block.header.state_root, state_root);

		// check extrinsics root
		let extrinsics = Self::get_state::<Vec<Vec<u8>>>(EXTRINSICS_KEY).unwrap_or_default();
		let extrinsics_root =
			BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);
		assert_eq!(block.header.extrinsics_root, extrinsics_root);

		info!(target: LOG_TARGET, "Finishing block import.");
		Self::print_state();
	}

	/// Apply a single extrinsic.
	///
	/// If an internal error occurs during the dispatch, such as "insufficient funds" etc, we don't
	/// care about which variant of `DispatchError` you return. But, if a bad signature is provided,
	/// then `Err(InvalidTransaction::BadProof)` must be returned.
	fn do_apply_extrinsic(ext: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
		info!(target: LOG_TARGET, "Entering apply_extrinsic: {:?}", ext);

		// TODO: we don't have a means of dispatch, implement it! You probably want to match on
		// `ext.function.call`, and start implementing different arms one at a time. Also, this is
		// called from both authoring and importing. It should "note" any extrinsic that
		// successfully executes in EXTRINSICS_KEY.

		// use shared::{RuntimeCall, SystemCall, VALUE_KEY};
		// match ext.function.call {
		// 	RuntimeCall::System(SystemCall::Set { value }) =>
		// 		sp_io::storage::set(&VALUE_KEY, &value.encode()),
		// 	_ => unimplemented!(),
		// }

		// for nor we note any extrinsic in the block, BUT BE AWARE, you should only do this if this
		// function is returning `Ok(_)`.
		// Self::mutate_state::<Vec<Vec<u8>>>(EXTRINSICS_KEY, |current| {
		// 	current.push(ext.encode());
		// });

		// The return type of this function, if called in the context of block authoring, means:
		// Ok(_) => in the block
		//   Ok(Ok(_)) => in the block, and succeed to do whatever it wanted to do
		//   Ok(Err(_)) => in the block, and failed to do whatever it wanted to do
		// Err(_) => not in the block

		// TODO: Need to use safe math for all arithmetic operations, but didn't have time to change
		// it       but I did some manual overflow checks.
		let payload = ext.function.encode();
		let signature = ext.signature.clone();
		let call = ext.function.call.clone();
		match signature {
			Some((address, signature, _)) => {
				if !sp_io::crypto::sr25519_verify(&signature, &payload, &address) {
					// bad signature
					Err(sp_runtime::transaction_validity::TransactionValidityError::Invalid(
						InvalidTransaction::BadProof,
					))
				} else {
					// good signature

					let account_storage_key =
						[b"BalancesMap", address.encode().as_slice()].concat();

					let mut account_balance =
						Self::get_state::<AccountBalance>(&account_storage_key).unwrap_or_default();

					//check if nonce is correct
					if account_balance.nonce < ext.function.nonce {
						// nonce is too high
						return Err(transaction_validity::TransactionValidityError::Invalid(
							InvalidTransaction::Future,
						))
					} else if account_balance.nonce > ext.function.nonce {
						// nonce is too low
						return Err(transaction_validity::TransactionValidityError::Invalid(
							InvalidTransaction::Stale,
						))
					}

					match call {
						RuntimeCall::Currency(currency_call) => match currency_call {
							CurrencyCall::Mint { dest, amount } => {
								// check if account has enough free balance to pay the tip
								if let Some(tip) = ext.function.tip {
									if account_balance.free - shared::MINIMUM_BALANCE < tip {
										// not enough free balance
										return Err(
											transaction_validity::TransactionValidityError::Invalid(
												InvalidTransaction::Payment,
											),
										)
									}
									// decrement the origin's free balance
									account_balance.free -= tip;
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);

									// update the treasury's storage with added tip
									Self::mutate_state(
										&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
											.concat(),
										|current: &mut AccountBalance| {
											if current.free + tip >= shared::MINIMUM_BALANCE {
												current.free += tip;
											} else {
												Self::mutate_state(
													b"TotalIssuance",
													|total_issuance: &mut u128| {
														*total_issuance -= tip;
													},
												)
											}
										},
									);
								}

								//get dest account balance
								let dest_storage_key =
									[b"BalancesMap", dest.encode().as_slice()].concat();

								let mut dest_balance =
									Self::get_state::<AccountBalance>(&dest_storage_key)
										.unwrap_or_default();

								//check for account balance overflow and TotalIssuance overflow
								let mut total_issuance =
									Self::get_state::<u128>(b"TotalIssuance").unwrap_or_default();

								if u128::MAX - dest_balance.free < amount ||
									u128::MAX - total_issuance < amount
								{
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// execution will cause an overflow
									return Ok(Err(DispatchError::Arithmetic(
										sp_runtime::ArithmeticError::Overflow,
									)))
								}

								// check if the free balance is below the minimum balance
								if dest_balance.free + amount < shared::MINIMUM_BALANCE {
									// not enough free balance
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									return Ok(Err(DispatchError::Other(
										"Free balance is below the minimum balance",
									)))
								}

								// check if the sender is SUDO
								if address != sp_core::sr25519::Public::from_raw(shared::SUDO) {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// dest does not have super user access to mint
									return Ok(Err(DispatchError::BadOrigin))
								}

								// increment the total issuance
								total_issuance += amount;

								if dest_storage_key == account_storage_key {
									// minting to self
									//increment account's free balance
									account_balance.free += amount;
								} else {
									//minting to others
									//increment the dest's free balance
									dest_balance.free += amount;
									sp_io::storage::set(&dest_storage_key, &dest_balance.encode());
								}

								// update the storage
								sp_io::storage::set(b"TotalIssuance", &total_issuance.encode());

								account_balance.nonce += 1;
								// update the storage for account
								sp_io::storage::set(
									&account_storage_key,
									&account_balance.encode(),
								);
							},
							CurrencyCall::Transfer { dest, amount } => {
								// check if account has enough free balance to pay the tip
								if let Some(tip) = ext.function.tip {
									if account_balance.free - shared::MINIMUM_BALANCE < tip {
										// not enough free balance
										return Err(
											transaction_validity::TransactionValidityError::Invalid(
												InvalidTransaction::Payment,
											),
										)
									}
									// decrement the origin's free balance
									account_balance.free -= tip;
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);

									// update the treasury's storage with added tip
									Self::mutate_state(
										&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
											.concat(),
										|current: &mut AccountBalance| {
											if current.free + tip >= shared::MINIMUM_BALANCE {
												current.free += tip;
											} else {
												Self::mutate_state(
													b"TotalIssuance",
													|total_issuance: &mut u128| {
														*total_issuance -= tip;
													},
												)
											}
										},
									);
								}

								//get dest account balance
								let dest_storage_key =
									[b"BalancesMap", dest.encode().as_slice()].concat();

								let mut dest_balance =
									Self::get_state::<AccountBalance>(&dest_storage_key)
										.unwrap_or_default();

								//check for account balance overflow
								if u128::MAX - dest_balance.free < amount {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// execution will cause an overflow
									return Ok(Err(DispatchError::Arithmetic(
										sp_runtime::ArithmeticError::Overflow,
									)))
								}

								// check if the free balance is below the minimum balance
								if account_balance.free - shared::MINIMUM_BALANCE < amount ||
									dest_balance.free + amount < shared::MINIMUM_BALANCE
								{
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// not enough free balance
									return Ok(Err(DispatchError::Other(
										"Free balance is below the minimum balance",
									)))
								}

								//check if transfer to self
								if account_storage_key != dest_storage_key {
									// decrement the origin's free balance
									account_balance.free -= amount;

									// increment the dest's free balance
									dest_balance.free += amount;

									sp_io::storage::set(&dest_storage_key, &dest_balance.encode());
								}

								account_balance.nonce += 1;
								// update the storage
								sp_io::storage::set(
									&account_storage_key,
									&account_balance.encode(),
								);
							},
							CurrencyCall::TransferAll { dest } => {
								// check if account has enough free balance to pay the tip
								if let Some(tip) = ext.function.tip {
									if account_balance.free - shared::MINIMUM_BALANCE < tip {
										// not enough free balance
										return Err(
											transaction_validity::TransactionValidityError::Invalid(
												InvalidTransaction::Payment,
											),
										)
									}
									// decrement the origin's free balance
									account_balance.free -= tip;
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);

									// update the treasury's storage with added tip
									Self::mutate_state(
										&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
											.concat(),
										|current: &mut AccountBalance| {
											if current.free + tip >= shared::MINIMUM_BALANCE {
												current.free += tip;
											} else {
												Self::mutate_state(
													b"TotalIssuance",
													|total_issuance: &mut u128| {
														*total_issuance -= tip;
													},
												)
											}
										},
									);
								}

								//get dest account balance
								let dest_storage_key =
									[b"BalancesMap", dest.encode().as_slice()].concat();

								let mut dest_balance =
									Self::get_state::<AccountBalance>(&dest_storage_key)
										.unwrap_or_default();

								// check if sender has reserved balance
								if account_balance.reserved > 0 {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// origin has reserved balance
									return Ok(Err(DispatchError::BadOrigin))
								}

								// check arithmatic overflow
								if u128::MAX - dest_balance.free < account_balance.free {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// execution will cause an overflow
									return Ok(Err(DispatchError::Arithmetic(
										sp_runtime::ArithmeticError::Overflow,
									)))
								}

								//check if transfer to self
								if dest_storage_key != account_storage_key {
									// increment the dest's free balance
									dest_balance.free += account_balance.free;

									// remove the origin's account
									sp_io::storage::clear(&account_storage_key);
									// update the storage
									sp_io::storage::set(&dest_storage_key, &dest_balance.encode());
								} else {
									account_balance.nonce += 1;
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);
								}
							},
						},
						RuntimeCall::Staking(staking_call) => match staking_call {
							StakingCall::Bond { amount } => {
								// check if account has enough free balance to pay the tip
								if let Some(tip) = ext.function.tip {
									if account_balance.free - shared::MINIMUM_BALANCE < tip {
										// not enough free balance
										return Err(
											transaction_validity::TransactionValidityError::Invalid(
												InvalidTransaction::Payment,
											),
										)
									}
									// decrement the origin's free balance
									account_balance.free -= tip;
									// update the storage
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);

									// update the treasury's storage with added tip
									Self::mutate_state(
										&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
											.concat(),
										|current: &mut AccountBalance| {
											if current.free + tip >= shared::MINIMUM_BALANCE {
												current.free += tip;
											} else {
												Self::mutate_state(
													b"TotalIssuance",
													|total_issuance: &mut u128| {
														*total_issuance -= tip;
													},
												)
											}
										},
									);
								}

								//check if the sender has enough free balance
								if account_balance.free - shared::MINIMUM_BALANCE < amount {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// not enough free balance
									return Ok(Err(DispatchError::BadOrigin))
								}

								//check for arithmetic overflow
								if u128::MAX - account_balance.reserved < amount {
									Self::mutate_state::<Vec<Vec<u8>>>(
										&EXTRINSICS_KEY,
										|extrinsics| extrinsics.push(ext.encode()),
									);
									// execution will cause an overflow
									return Ok(Err(DispatchError::Arithmetic(
										sp_runtime::ArithmeticError::Overflow,
									)))
								}

								// decrement the origin's free balance
								account_balance.free -= amount;

								// increment the origin's reserved balance
								account_balance.reserved += amount;

								account_balance.nonce += 1;

								// update the storage
								sp_io::storage::set(
									&account_storage_key,
									&account_balance.encode(),
								);
							},
						},
						RuntimeCall::System(system_call) => {
							match system_call {
								SystemCall::Remark { .. } => {
									// check if account has enough free balance to pay the tip
									if let Some(tip) = ext.function.tip {
										if account_balance.free - shared::MINIMUM_BALANCE < tip {
											// not enough free balance
											return Err(transaction_validity::TransactionValidityError::Invalid(
								InvalidTransaction::Payment,
							))
										}
										// decrement the origin's free balance
										account_balance.free -= tip;
										// update the storage
										sp_io::storage::set(
											&account_storage_key,
											&account_balance.encode(),
										);

										// update the treasury's storage with added tip
										Self::mutate_state(
											&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
												.concat(),
											|current: &mut AccountBalance| {
												if current.free + tip >= shared::MINIMUM_BALANCE {
													current.free += tip;
												} else {
													Self::mutate_state(
														b"TotalIssuance",
														|total_issuance: &mut u128| {
															*total_issuance -= tip;
														},
													)
												}
											},
										);
									}

									account_balance.nonce += 1;

									// update the storage
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);
								},
								SystemCall::SudoRemark { .. } => {
									// check if account has enough free balance to pay the tip
									if let Some(tip) = ext.function.tip {
										if account_balance.free - shared::MINIMUM_BALANCE < tip {
											// not enough free balance
											return Err(transaction_validity::TransactionValidityError::Invalid(
								InvalidTransaction::Payment,
							))
										}
										// decrement the origin's free balance
										account_balance.free -= tip;
										// update the storage
										sp_io::storage::set(
											&account_storage_key,
											&account_balance.encode(),
										);

										// update the treasury's storage with added tip
										Self::mutate_state(
											&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
												.concat(),
											|current: &mut AccountBalance| {
												if current.free + tip >= shared::MINIMUM_BALANCE {
													current.free += tip;
												} else {
													Self::mutate_state(
														b"TotalIssuance",
														|total_issuance: &mut u128| {
															*total_issuance -= tip;
														},
													)
												}
											},
										);
									}

									// check if the sender is SUDO
									if address != sp_core::sr25519::Public::from_raw(shared::SUDO) {
										// dest does not have super user access to mint
										Self::mutate_state::<Vec<Vec<u8>>>(
											&EXTRINSICS_KEY,
											|extrinsics| extrinsics.push(ext.encode()),
										);
										return Ok(Err(DispatchError::BadOrigin))
									}

									account_balance.nonce += 1;

									// update the storage
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);
								},
								SystemCall::Set { value } => {
									// check if account has enough free balance to pay the tip
									if let Some(tip) = ext.function.tip {
										if account_balance.free - shared::MINIMUM_BALANCE < tip {
											// not enough free balance
											return Err(transaction_validity::TransactionValidityError::Invalid(
								InvalidTransaction::Payment,
							))
										}
										// decrement the origin's free balance
										account_balance.free -= tip;
										// update the storage
										sp_io::storage::set(
											&account_storage_key,
											&account_balance.encode(),
										);

										// update the treasury's storage with added tip
										Self::mutate_state(
											&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
												.concat(),
											|current: &mut AccountBalance| {
												if current.free + tip >= shared::MINIMUM_BALANCE {
													current.free += tip;
												} else {
													Self::mutate_state(
														b"TotalIssuance",
														|total_issuance: &mut u128| {
															*total_issuance -= tip;
														},
													)
												}
											},
										);
									}

									// set the value in storage.
									sp_io::storage::set(&VALUE_KEY, &value.encode());

									account_balance.nonce += 1;

									// update the storage
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);
								},
								SystemCall::Upgrade { code } => {
									// check if account has enough free balance to pay the tip
									if let Some(tip) = ext.function.tip {
										if account_balance.free - shared::MINIMUM_BALANCE < tip {
											// not enough free balance
											return Err(transaction_validity::TransactionValidityError::Invalid(
								InvalidTransaction::Payment,
							))
										}
										// decrement the origin's free balance
										account_balance.free -= tip;
										sp_io::storage::set(
											&account_storage_key,
											&account_balance.encode(),
										);

										// update the treasury's storage with added tip
										Self::mutate_state(
											&[b"BalancesMap", shared::TREASURY.encode().as_slice()]
												.concat(),
											|current: &mut AccountBalance| {
												if current.free + tip >= shared::MINIMUM_BALANCE {
													current.free += tip;
												} else {
													Self::mutate_state(
														b"TotalIssuance",
														|total_issuance: &mut u128| {
															*total_issuance -= tip;
														},
													)
												}
											},
										);
									}

									// check if the sender is SUDO
									if address != sp_core::sr25519::Public::from_raw(shared::SUDO) {
										// dest does not have super user access to mint
										Self::mutate_state::<Vec<Vec<u8>>>(
											&EXTRINSICS_KEY,
											|extrinsics| extrinsics.push(ext.encode()),
										);
										return Ok(Err(DispatchError::BadOrigin))
									}

									// set the code in storage.
									sp_io::storage::set(b":code", &code);

									account_balance.nonce += 1;
									// update the storage
									sp_io::storage::set(
										&account_storage_key,
										&account_balance.encode(),
									);
								},
							}
						},
					}

					// ext execution successful, add to extrinsics.
					Self::mutate_state::<Vec<Vec<u8>>>(&EXTRINSICS_KEY, |extrinsics| {
						extrinsics.push(ext.encode())
					});
					info!(target: LOG_TARGET, "Finishing applying extrinsic.");
					Self::print_state();

					Ok(Ok(()))
				}
			},
			// no signature, invalid transaction
			None => Err(sp_runtime::transaction_validity::TransactionValidityError::Invalid(
				InvalidTransaction::BadProof,
			)),
		}
	}

	fn do_validate_transaction(
		_source: TransactionSource,
		ext: <Block as BlockT>::Extrinsic,
		_block_hash: <Block as BlockT>::Hash,
	) -> TransactionValidity {
		log::debug!(target: LOG_TARGET,"Entering validate_transaction. tx: {:?}", ext);

		// TODO: we don't have a means of validating, implement it!
		// NOTE: every transaction must provide _something_, we provide a dummy value here.

		info!(target: LOG_TARGET, "Finishing validating transaction.");
		Self::print_state();

		let payload = ext.function.encode();
		match ext.signature {
			Some((address, signature, _)) => {
				if !sp_io::crypto::sr25519_verify(&signature, &payload, &address) {
					// bad signature
					Err(sp_runtime::transaction_validity::TransactionValidityError::Invalid(
						sp_runtime::transaction_validity::InvalidTransaction::BadProof,
					))
				} else {
					// good signature
					let account_storage_key =
						[b"BalancesMap", address.encode().as_slice()].concat();
					let account_balance =
						Self::get_state::<AccountBalance>(&account_storage_key).unwrap_or_default();

					//initialize a valid transaction
					let mut valid_transaction = ValidTransaction::default();

					// check if account has enough free balance for tip
					if let Some(tip) = ext.function.tip {
						if account_balance.free - shared::MINIMUM_BALANCE < tip {
							// not enough free balance
							return Err(
								sp_runtime::transaction_validity::TransactionValidityError::Invalid(
									InvalidTransaction::Payment,
								),
							)
						}
						valid_transaction.priority = u64::try_from(tip).unwrap_or(u64::MAX);
					}

					// check if the ext has the correct nonce
					if ext.function.nonce < account_balance.nonce {
						// nonce is too low
						return Err(
							sp_runtime::transaction_validity::TransactionValidityError::Invalid(
								InvalidTransaction::Stale,
							),
						)
					}
					Ok(valid_transaction)
				}
			},
			// no signature, invalid transaction
			None => Err(sp_runtime::transaction_validity::TransactionValidityError::Invalid(
				sp_runtime::transaction_validity::InvalidTransaction::BadProof,
			)),
		}
	}
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Self::do_execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Self::do_initialize_block(header)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Self::do_apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Self::do_finalize_block()
		}

		fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			Default::default()
		}

		fn check_inherents(
			_block: Block,
			_data: sp_inherents::InherentData
		) -> sp_inherents::CheckInherentsResult {
			Default::default()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Self::do_validate_transaction(source, tx, block_hash)
		}
	}

	// Ignore everything after this.

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Default::default())
		}

		fn metadata_at_version(_version: u32) -> Option<OpaqueMetadata> {
			Default::default()
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Default::default()
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(_header: &<Block as BlockT>::Header) {}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(_: Option<Vec<u8>>) -> Vec<u8> {
			Default::default()
		}

		fn decode_session_keys(
			_: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			Default::default()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::RuntimeCallExt;
	use parity_scale_codec::Encode;
	use shared::{Extrinsic, RuntimeCall, VALUE_KEY};
	use sp_core::hexdisplay::HexDisplay;
	use sp_io::TestExternalities;
	use sp_runtime::{
		traits::Extrinsic as _,
		transaction_validity::{InvalidTransaction, TransactionValidityError},
	};

	fn set_value_call(value: u32, nonce: u32) -> RuntimeCallExt {
		RuntimeCallExt {
			call: RuntimeCall::System(shared::SystemCall::Set { value }),
			tip: None,
			nonce,
		}
	}

	fn unsigned_set_value(value: u32) -> Extrinsic {
		let call = RuntimeCallExt {
			call: RuntimeCall::System(shared::SystemCall::Set { value }),
			tip: None,
			nonce: 0,
		};
		Extrinsic::new(call, None).unwrap()
	}

	fn signed_set_value(value: u32, nonce: u32) -> Extrinsic {
		let call = set_value_call(value, nonce);
		let signer = sp_keyring::AccountKeyring::Alice;
		let payload = call.encode();
		let signature = signer.sign(&payload);
		Extrinsic::new(call, Some((signer.public(), signature, ()))).unwrap()
	}

	/// Return the list of extrinsics that are noted in the `EXTRINSICS_KEY`.
	fn noted_extrinsics() -> Vec<Vec<u8>> {
		sp_io::storage::get(EXTRINSICS_KEY)
			.and_then(|bytes| <Vec<Vec<u8>> as Decode>::decode(&mut &*bytes).ok())
			.unwrap_or_default()
	}

	#[test]
	fn does_it_print() {
		// runt this with `cargo test does_it_print -- --nocapture`
		println!("{:?}", ValidTransaction::default());
	}

	#[test]
	fn does_it_log() {
		// run this with RUST_LOG=frameless=trace cargo test -p runtime does_it_log
		sp_tracing::try_init_simple();
		log::info!(target: LOG_TARGET, "Something");
	}

	#[docify::export]
	#[test]
	fn host_function_call_works() {
		// this is just to demonstrate to you that you should always wrap any code containing host
		// functions in `TestExternalities`.
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::get(&VALUE_KEY);
		})
	}

	#[docify::export]
	#[test]
	fn encode_examples() {
		// demonstrate some basic encodings. Example usage:
		//
		// ```
		// wscat -c 127.0.0.1:9944 -x '{"jsonrpc":"2.0", "id":1, "method":"state_getStorage", "params": ["0x123"]}'
		// wscat -c ws://127.0.0.1:9944 -x '{"jsonrpc":"2.0", "id":1, "method":"author_submitExtrinsic", "params": ["0x123"]}'
		// ```
		let unsigned = Extrinsic::new_unsigned(set_value_call(42, 0));

		let signer = sp_keyring::AccountKeyring::Alice;
		let call = set_value_call(42, 0);
		let payload = (call).encode();
		let signature = signer.sign(&payload);
		let signed = Extrinsic::new(call, Some((signer.public(), signature, ()))).unwrap();

		println!("unsigned = {:?} {:?}", unsigned, HexDisplay::from(&unsigned.encode()));
		println!("signed {:?} {:?}", signed, HexDisplay::from(&signed.encode()));
		println!("value key = {:?}", HexDisplay::from(&VALUE_KEY));
	}

	#[docify::export]
	#[test]
	fn signed_set_value_works() {
		// A signed `Set` works.
		let ext = signed_set_value(42, 0);
		TestExternalities::new_empty().execute_with(|| {
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(noted_extrinsics().len(), 0);

			Runtime::do_apply_extrinsic(ext).unwrap().unwrap();

			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), Some(42));
			assert_eq!(noted_extrinsics().len(), 1, "transaction should have been noted!");
		});
	}

	#[docify::export]
	#[test]
	fn bad_signature_fails() {
		// A poorly signed extrinsic must fail.
		let signer = sp_keyring::AccountKeyring::Alice;
		let call = set_value_call(42, 0);
		let bad_call = set_value_call(43, 0);
		let payload = (bad_call).encode();
		let signature = signer.sign(&payload);
		let ext = Extrinsic::new(call, Some((signer.public(), signature, ()))).unwrap();

		TestExternalities::new_empty().execute_with(|| {
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(
				Runtime::do_apply_extrinsic(ext).unwrap_err(),
				TransactionValidityError::Invalid(InvalidTransaction::BadProof)
			);
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(noted_extrinsics().len(), 0, "transaction should have not been noted!");
		});
	}

	#[docify::export]
	#[test]
	fn unsigned_set_value_does_not_work() {
		// An unsigned `Set` must fail as well.
		let ext = unsigned_set_value(42);

		TestExternalities::new_empty().execute_with(|| {
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(
				Runtime::do_apply_extrinsic(ext).unwrap_err(),
				TransactionValidityError::Invalid(InvalidTransaction::BadProof)
			);
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(noted_extrinsics().len(), 0);
		});
	}

	#[docify::export]
	#[test]
	fn validate_works() {
		// An unsigned `Set` cannot be validated. Same should go for one with a bad signature.
		let ext = unsigned_set_value(42);

		TestExternalities::new_empty().execute_with(|| {
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
			assert_eq!(
				Runtime::do_validate_transaction(
					TransactionSource::External,
					ext,
					Default::default()
				)
				.unwrap_err(),
				TransactionValidityError::Invalid(InvalidTransaction::BadProof)
			);
			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), None);
		});
	}

	#[docify::export]
	#[test]
	fn import_and_author_equal() {
		// a few dummy extrinsics.
		let ext1 = signed_set_value(42, 0);
		let ext2 = signed_set_value(43, 1);
		let ext3 = signed_set_value(44, 2);

		let header = shared::Header {
			digest: Default::default(),
			extrinsics_root: Default::default(),
			parent_hash: Default::default(),
			number: 0,
			state_root: Default::default(),
		};

		// authoring a block:
		let block = TestExternalities::new_empty().execute_with(|| {
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext1.clone()).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext2.clone()).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext3.clone()).unwrap().unwrap();

			let header = Runtime::do_finalize_block();

			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);
			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 3, "incorrect extrinsics_key recorded in state");

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);

			Block { extrinsics: vec![ext1, ext2, ext3], header }
		});

		// now re-importing it.
		TestExternalities::new_empty().execute_with(|| {
			// This should internally check state/extrinsics root. If it does not panic, then we are
			// gucci
			Runtime::do_execute_block(block.clone());

			assert_eq!(Runtime::get_state::<u32>(VALUE_KEY), Some(44));

			// double check the extrinsic and state root:
			assert_eq!(
				block.header.state_root,
				H256::decode(&mut &sp_io::storage::root(Default::default())[..][..]).unwrap(),
				"incorrect state root in authored block after importing"
			);
			assert_eq!(
				block.header.extrinsics_root,
				BlakeTwo256::ordered_trie_root(
					block.extrinsics.into_iter().map(|e| e.encode()).collect::<Vec<_>>(),
					sp_runtime::StateVersion::V0
				),
				"incorrect extrinsics root in authored block",
			);
		});
	}

	#[test]
	fn storage_keys_sanity_check() {
		// Use your abstractions to fetch the final storage key for alice and total issuance. The
		// given bytes should act as the source of truth.
		let alice_balance_raw_key: Vec<u8> =
			[b"BalancesMap", sp_keyring::AccountKeyring::Alice.public().encode().as_slice()]
				.concat();
		let total_issuance_raw_key: Vec<u8> = b"TotalIssuance".to_vec();

		assert_eq!(
			alice_balance_raw_key,
			vec![
				66, 97, 108, 97, 110, 99, 101, 115, 77, 97, 112, 212, 53, 147, 199, 21, 253, 211,
				28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154,
				86, 132, 231, 165, 109, 162, 125,
			]
		);

		assert_eq!(
			total_issuance_raw_key,
			vec![84, 111, 116, 97, 108, 73, 115, 115, 117, 97, 110, 99, 101]
		);
	}

	//>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>Additional Tests>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
	#[docify::export]
	#[test]
	fn alice_can_mint_to_bob() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", alice.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);
			sp_io::storage::set(b"TotalIssuance", &100u128.encode());
			let ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: bob.public(),
						amount: 20,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: bob.public(),
								amount: 20,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext).unwrap().unwrap();
			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 1, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 20, reserved: 0, nonce: 0 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 120);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn bob_cannot_mint_to_alice() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", bob.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);

			sp_io::storage::set(b"TotalIssuance", &100u128.encode());

			sp_io::storage::set(&EXTRINSICS_KEY, &vec![vec![(); 0]].encode());
			let ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: alice.public(),
						amount: 20,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					bob.public(),
					bob.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: alice.public(),
								amount: 20,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			println!("ext = {:?}", ext);

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext).unwrap().unwrap_err();
			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 1, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", alice.public().encode().as_slice()].concat()
				),
				None
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 100);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn validate_signed_set_value_okay() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let ext = Extrinsic::new(
			RuntimeCallExt {
				call: RuntimeCall::System(shared::SystemCall::Set { value: 42 }),
				nonce: 0,
				tip: None,
			},
			Some((
				alice.public(),
				alice.sign(
					&RuntimeCallExt {
						call: RuntimeCall::System(shared::SystemCall::Set { value: 42 }),
						nonce: 0,
						tip: None,
					}
					.encode(),
				),
				(),
			)),
		)
		.unwrap();

		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", alice.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);
			assert_eq!(
				Runtime::do_validate_transaction(
					TransactionSource::External,
					ext,
					Default::default()
				),
				Ok(ValidTransaction::default())
			);
		});
	}

	#[docify::export]
	#[test]
	fn validate_sudo_remark_by_bob() {
		let bob = sp_keyring::AccountKeyring::Bob;
		let ext = Extrinsic::new(
			RuntimeCallExt {
				call: RuntimeCall::System(shared::SystemCall::SudoRemark {
					data: b"Hello".to_vec(),
				}),
				nonce: 0,
				tip: None,
			},
			Some((
				bob.public(),
				bob.sign(
					&RuntimeCallExt {
						call: RuntimeCall::System(shared::SystemCall::SudoRemark {
							data: b"Hello".to_vec(),
						}),
						nonce: 0,
						tip: None,
					}
					.encode(),
				),
				(),
			)),
		)
		.unwrap();

		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", bob.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);

			assert_eq!(
				Runtime::do_validate_transaction(
					TransactionSource::External,
					ext.clone(),
					Default::default()
				),
				Ok(ValidTransaction::default())
			);

			assert_eq!(Runtime::do_apply_extrinsic(ext).unwrap(), Err(DispatchError::BadOrigin));
		});
	}

	#[docify::export]
	#[test]
	fn alice_mints_100_to_bob_bob_transfers_100_to_alice() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", alice.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);

			sp_io::storage::set(
				&[b"BalancesMap", bob.public().encode().as_slice()].concat(),
				&AccountBalance { free: 10, reserved: 0, nonce: 0 }.encode(),
			);

			sp_io::storage::set(b"TotalIssuance", &110u128.encode());
			let ext_mint = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: bob.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: bob.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let ext_transfer = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Transfer {
						dest: alice.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					bob.public(),
					bob.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Transfer {
								dest: alice.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext_mint).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext_transfer).unwrap().unwrap();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 2, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 10, reserved: 0, nonce: 1 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", alice.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 200, reserved: 0, nonce: 1 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 210);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn multiple_mints_in_single_block() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		let charlie = sp_keyring::AccountKeyring::Charlie;

		TestExternalities::new_empty().execute_with(|| {
			let alice_to_bob_ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: bob.public(),
						amount: 20,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: bob.public(),
								amount: 20,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let alice_to_alice_ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: alice.public(),
						amount: 30,
					}),
					nonce: 1,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: alice.public(),
								amount: 30,
							}),
							nonce: 1,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let alice_to_charlie_ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: charlie.public(),
						amount: 50,
					}),
					nonce: 2,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: charlie.public(),
								amount: 50,
							}),
							nonce: 2,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(alice_to_bob_ext).unwrap().unwrap();
			Runtime::do_apply_extrinsic(alice_to_alice_ext).unwrap().unwrap();
			Runtime::do_apply_extrinsic(alice_to_charlie_ext).unwrap().unwrap();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 3, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 20, reserved: 0, nonce: 0 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", alice.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 30, reserved: 0, nonce: 3 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", charlie.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 50, reserved: 0, nonce: 0 })
			);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn alice_with_100_tips_105() {
		let alice = sp_keyring::AccountKeyring::Alice;

		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(
				&[b"BalancesMap", alice.public().encode().as_slice()].concat(),
				&AccountBalance { free: 100, reserved: 0, nonce: 0 }.encode(),
			);

			let ext = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::System(shared::SystemCall::Set { value: 42 }),
					nonce: 0,
					tip: Some(105u128),
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::System(shared::SystemCall::Set { value: 42 }),
							nonce: 0,
							tip: Some(105u128),
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			Runtime::do_validate_transaction(TransactionSource::External, ext, Default::default())
				.unwrap_err();
		});
	}

	#[docify::export]
	#[test]
	fn alice_with_100_transfers_20_to_bob_with_10_tip() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(b"TotalIssuance", &0u128.encode());
			let ext_mint = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: alice.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: alice.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let ext_transfer = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Transfer {
						dest: bob.public(),
						amount: 20,
					}),
					nonce: 1,
					tip: Some(10),
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Transfer {
								dest: bob.public(),
								amount: 20,
							}),
							nonce: 1,
							tip: Some(10),
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext_mint).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext_transfer).unwrap().unwrap();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 2, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 20, reserved: 0, nonce: 0 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", alice.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 70, reserved: 0, nonce: 2 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", shared::TREASURY.encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 10, reserved: 0, nonce: 0 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 100);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn alice_with_100_transfers_all_to_bob_and_tips_5() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(b"TotalIssuance", &0u128.encode());
			let ext_mint = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: alice.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: alice.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let ext_transfer_all = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::TransferAll {
						dest: bob.public(),
					}),
					nonce: 1,
					tip: Some(5),
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::TransferAll {
								dest: bob.public(),
							}),
							nonce: 1,
							tip: Some(5),
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext_mint).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext_transfer_all).unwrap().unwrap();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 2, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 95, reserved: 0, nonce: 0 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", alice.public().encode().as_slice()].concat()
				),
				None
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", shared::TREASURY.encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 0, reserved: 0, nonce: 0 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 95);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn alice_mints_100_to_bob_bob_stakes_85_and_tip_10() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(b"TotalIssuance", &0u128.encode());
			let ext_mint = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: bob.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: bob.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let ext_staking = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Staking(shared::StakingCall::Bond { amount: 85 }),
					nonce: 0,
					tip: Some(10),
				},
				Some((
					bob.public(),
					bob.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Staking(shared::StakingCall::Bond { amount: 85 }),
							nonce: 0,
							tip: Some(10),
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext_mint).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext_staking).unwrap().unwrap_err();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 2, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 90, reserved: 0, nonce: 0 })
			);

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", shared::TREASURY.encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 10, reserved: 0, nonce: 0 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 100);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}

	#[docify::export]
	#[test]
	fn alice_mints_100_to_bob_bob_stakes_120() {
		let alice = sp_keyring::AccountKeyring::Alice;
		let bob = sp_keyring::AccountKeyring::Bob;
		TestExternalities::new_empty().execute_with(|| {
			sp_io::storage::set(b"TotalIssuance", &0u128.encode());
			let ext_mint = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
						dest: bob.public(),
						amount: 100,
					}),
					nonce: 0,
					tip: None,
				},
				Some((
					alice.public(),
					alice.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Currency(shared::CurrencyCall::Mint {
								dest: bob.public(),
								amount: 100,
							}),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let ext_staking = Extrinsic::new(
				RuntimeCallExt {
					call: RuntimeCall::Staking(shared::StakingCall::Bond { amount: 120 }),
					nonce: 0,
					tip: None,
				},
				Some((
					bob.public(),
					bob.sign(
						&RuntimeCallExt {
							call: RuntimeCall::Staking(shared::StakingCall::Bond { amount: 120 }),
							nonce: 0,
							tip: None,
						}
						.encode(),
					),
					(),
				)),
			)
			.unwrap();

			let header = shared::Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				parent_hash: Default::default(),
				number: 0,
				state_root: Default::default(),
			};
			Runtime::do_initialize_block(&header);
			drop(header);

			Runtime::do_apply_extrinsic(ext_mint).unwrap().unwrap();
			Runtime::do_apply_extrinsic(ext_staking).unwrap().unwrap_err();

			let header = Runtime::do_finalize_block();
			assert!(
				sp_io::storage::get(HEADER_KEY).is_none(),
				"header must have been cleared from storage"
			);

			let extrinsics = noted_extrinsics();
			assert_eq!(extrinsics.len(), 2, "incorrect extrinsics_key recorded in state");

			assert_eq!(
				Runtime::get_state::<AccountBalance>(
					&[b"BalancesMap", bob.public().encode().as_slice()].concat()
				),
				Some(AccountBalance { free: 100, reserved: 0, nonce: 0 })
			);

			assert_eq!(Runtime::get_state::<u128>(b"TotalIssuance").unwrap(), 100);

			let expected_state_root = {
				let raw_state_root = &sp_io::storage::root(Default::default())[..];
				H256::decode(&mut &raw_state_root[..]).unwrap()
			};
			let expected_extrinsics_root =
				BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);

			assert_eq!(
				header.state_root, expected_state_root,
				"block finalization should set correct state root in header"
			);
			assert_eq!(
				header.extrinsics_root, expected_extrinsics_root,
				"block finalization should set correct extrinsics root in header"
			);
		});
	}
}
