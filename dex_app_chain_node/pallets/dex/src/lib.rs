#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
pub use pallet::*;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
	ensure,
	traits::{
		tokens::{AssetId, Balance, Precision::Exact},
		Incrementable,
	},
};
use frame_system::{
	ensure_root, ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
pub use pallet::*;
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, TrailingZeroInput};
pub use types::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		traits::{
			fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
			fungibles::{Create, Destroy, Inspect, Mutate},
			tokens::{Fortitude::Polite, Preservation::Expendable},
		},
		Hashable,
	};
	use sp_runtime::traits::{IntegerSquareRoot, One, Zero};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// type for native asset (token) of this chain.
		type NativeAsset: InspectFungible<Self::AccountId, Balance = Self::NativeBalance>
			+ MutateFungible<Self::AccountId>;

		// type for native asset balance.
		type NativeBalance: Balance;

		// type for non-native asset balance.
		type AssetBalance: Balance;

		// type for identifying non-native assets.
		type AssetId: AssetId + Ord;

		// type for identifying a Lp token.
		type LpTokenId: AssetId + Ord + Incrementable + From<u32>;

		// registry for supported non-native assets.
		type AssetsRegistry: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId>;

		// registry for Lp tokens.
		type LpAssetsRegistry: Inspect<Self::AccountId, AssetId = Self::LpTokenId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Destroy<Self::AccountId>;

		// type for the current block number.
		type CurrentBlockNumber: Get<BlockNumberFor<Self>>;

		/// a % the liquidity providers will take of every swap. Represents 0.1%.
		#[pallet::constant]
		type SwapFee: Get<u32>;

		// type for pool setup deposit to incentivize cleaning up unused pools.
		#[pallet::constant]
		type PoolSetupDeposit: Get<Self::NativeBalance>;
	}

	/// Map from `PoolAssetId` to `PoolInfo`. This establishes whether a pool has been officially
	/// created.
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_128Concat, PoolIdOf<T>, PoolInfo<T::LpTokenId>, OptionQuery>;

	/// Stores the `PoolAssetId` that is going to be used for the next lp token.
	/// This gets incremented whenever a new lp pool is created.
	#[pallet::storage]
	#[pallet::getter(fn next_lp_token_id)]
	pub type NextLpTokenId<T: Config> = StorageValue<_, T::LpTokenId, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The event emitted when a new pool is created.
		PoolCreated {
			/// The account that created the pool.
			creator: T::AccountId,
			/// The pool id of the pool (a tuple of the a asset id pair sorted
			/// using scale encoding in ascending order).
			pool_id: PoolIdOf<T>,
			/// The account ID of the pool.
			pool_account: T::AccountId,
			/// The id of the liquidity tokens that will be minted when liquidity is added to this
			/// pool.
			lp_token: T::LpTokenId,
		},

		/// The event emitted when liquidity is added to a pool.
		LiquidityAdded {
			/// The account id of the liquidity provider.
			liquidity_provider: T::AccountId,
			/// The pool id of the pool that the liquidity was added to.
			pool_id: PoolIdOf<T>,
			/// The amount of the first asset that was added to the pool.
			asset1_amount_provided: T::AssetBalance,
			/// The amount of the second asset that was added to the pool.
			asset2_amount_provided: T::AssetBalance,
			/// The id of the lp token that was minted.
			lp_token: T::LpTokenId,
			/// The amount of lp tokens that were minted.
			lp_token_amount_minted: T::AssetBalance,
		},

		/// The event emitted when liquidity is removed from a pool.
		LiquidityRemoved {
			/// The account id of the liquidity provider.
			remover: T::AccountId,
			/// The pool id of the pool that the liquidity was removed from.
			pool_id: PoolIdOf<T>,
			/// The amount of the first asset that was received.
			asset1_received_amount: T::AssetBalance,
			/// The amount of the second asset that was received.
			asset2_received_amount: T::AssetBalance,
			/// The id of the lp token that was burned.
			lp_token: T::LpTokenId,
			/// The amount of lp tokens that were burned.
			lp_token_burned: T::AssetBalance,
		},

		/// The event emitted when a pool is destroyed.
		PoolDestroyed {
			/// The account id of the liquidity provider.
			destroyer: T::AccountId,
			/// The pool id of the pool that was destroyed.
			pool_id: PoolIdOf<T>,
			/// The account ID of the pool.
			pool_account: T::AccountId,
			/// The LP token id of the pool.
			lp_token: T::LpTokenId,
		},

		/// The event emitted when a price oracle is called.
		PriceInfo {
			/// The account id of the price querier.
			querier: T::AccountId,
			/// The pool id of the asset pair
			pool_id: PoolIdOf<T>,
			/// The amount of the querying asset.
			asset_amount: T::AssetBalance,
			/// The amount of the querying asset in reserve.
			asset_pool_reserve: T::AssetBalance,
			/// The amount of the unit asset in reserve.
			price_unit_pool_reserve: T::AssetBalance,
			/// marginal_price = asset_amount * asset_pool_reserve / asset_unit_pool_reserve
			marginal_price: T::AssetBalance,
		},

		/// The event emitted when a swap is successful.
		SwapSucceeded {
			/// The account id of the swapper.
			user: T::AccountId,
			/// The asset id of the asset that was swapped in.
			asset_in: T::AssetId,
			/// The asset id of the asset that was swapped out.
			asset_out: T::AssetId,
			/// The amount of the asset that was swapped in.
			amount_in: T::AssetBalance,
			/// The amount of the asset that was swapped out.
			amount_out: T::AssetBalance,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Cannot create pool with same asset.
		CannotCreatePoolWithSameAsset,
		/// Pool already exists.
		PoolAlreadyExists,
		/// Incorrect LP token id.
		IncorrectLpTokenId,
		/// Invalid liquidity amount for an asset.
		InvalidLiquidityAmount,
		/// Pool not found.
		PoolNotFound,
		/// Passed the deadline set for the transaction.
		DeadlinePassed,
		/// Overflow when doing arithmetic operations.
		ArithmeticOverflow,
		/// Not enough liquidity provided.
		NotEnoughLiquidityProvided,
		/// Cannot add liquidity with same asset.
		CannotAddLiquidityWithSameAsset,
		/// Add liquidity failed.
		AddLiquidityFailed,
		/// Not enough liquidity token.
		NotEnoughLiquidityToken,
		/// Remove liquidity did not meet minimum amount.
		RemoveLiquidityDidNotMeetMinimumAmount,
		/// Cannot destroy non-empty pool.
		CannotDestroyPoolWithLiquidity,
		/// Empty pool.
		EmptyPool,
		/// Zero amount.
		CannotSwapZeroAmount,
		/// Cannot swap same asset.
		CannotSwapSameAsset,
		/// Insufficient minimum out for swap.
		InsufficientMinimumForSwap,
		/// Amount out too high.
		AmountOutTooHigh,
		/// Insufficient maximum in for swap.
		InsufficientMaximumForSwap,
		/// Not enough balance.
		AmountMoreThanBalance,
		/// Sender does not have enough native asset balance to pay for pool setup deposit
		NotEnoughToPayForPoolSetupDeposit,
		/// Cannot mint zero tokens to an account.
		CannotMintZeroAmount,
		/// Cannot redeem lp token more than its total supply
		CannotRedeemMoreThanTotalSupply,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new liquidity pool and provide initial liquidity. The creator will need to make
		/// a pool setup deposit in native token to incentivize cleaning up unused pools.
		///
		/// **parameters**
		/// - `origin`: The account that is creating the pool.
		/// - `asset1`: The first asset to be added to the pool.
		/// - `asset2`: The second asset to be added to the pool.
		/// - `amount1`: The amount of the first asset to be added to the pool.
		/// - `amount2`: The amount of the second asset to be added to the pool.
		/// - `min_lp_token_amount`: The minimum amount of liquidity token that should be minted.
		///
		/// **errors**
		/// - `CannotCreatePoolWithSameAsset`: Cannot create pool with same asset.
		/// - `PoolAlreadyExists`: Pool already exists.
		/// - `IncorrectLpTokenId`: Incorrect LP token id.
		/// - `InvalidLiquidityAmount`: Invalid liquidity amount for an asset.
		/// - `NotEnoughToPayForPoolSetupDeposit`: Sender does not have enough native asset balance
		/// - `AmountMoreThanBalance`: Not enough balance.
		/// - `AddLiquidityFailed`: Add liquidity failed.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset1: T::AssetId,
			asset2: T::AssetId,
			amount1: T::AssetBalance,
			amount2: T::AssetBalance,
			min_lp_token_amount: T::AssetBalance,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let sender = ensure_signed(origin)?;
			ensure!(asset1 != asset2, Error::<T>::CannotCreatePoolWithSameAsset);

			// Get a pool id
			// Pool id is a tuple of the two given asset ids sorted using scale encoding in
			// ascending order.
			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());
			ensure!(!Pools::<T>::contains_key(&pool_id), Error::<T>::PoolAlreadyExists);

			// Create a pool account by hashing the scaled encoding of the pool id.
			let pool_account = Self::get_pool_account(&pool_id);
			frame_system::Pallet::<T>::inc_providers(&pool_account);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Create Liquidity Token >>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			// Get the id for the next liquidity token.
			let lp_token = NextLpTokenId::<T>::get()
				.or(Some(T::LpTokenId::initial_value()))
				.ok_or(Error::<T>::IncorrectLpTokenId)?;

			// Create the liquidity token .
			T::LpAssetsRegistry::create(lp_token.clone(), pool_account.clone(), false, One::one())?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Update Storage >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			Pools::<T>::insert(pool_id.clone(), PoolInfo { lp_token: lp_token.clone() });

			NextLpTokenId::<T>::set(Some(lp_token.increment()));

			// >>>>>>>>>>>>>>>>>> Add Initial Liquidity & Setup Deposit >>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::do_add_liquidity(
				&sender,
				asset1.clone(),
				asset2.clone(),
				amount1,
				amount2,
				min_lp_token_amount,
			)?;

			// Transfer native asset to pool account as pool deposit.
			//
			// This pool deposit incentivize cleaning up unused pools, and will be refunded to
			// whoever destroys the pool when the pool has no liquidity left.
			ensure!(
				<<T as Config>::NativeAsset>::reducible_balance(&sender, Expendable, Polite) >=
					T::PoolSetupDeposit::get(),
				Error::<T>::NotEnoughToPayForPoolSetupDeposit
			);
			T::NativeAsset::transfer(
				&sender,
				&pool_account,
				T::PoolSetupDeposit::get(),
				Expendable,
			)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::PoolCreated {
				creator: sender,
				pool_id,
				pool_account,
				lp_token,
			});

			Ok(())
		}

		/// Add liquidity to an existing liquidity pool.
		///
		/// **parameters**
		/// - `origin`: The account that is adding liquidity to the pool.
		/// - `asset1`: The first asset to be added to the pool.
		/// - `asset2`: The second asset to be added to the pool.
		/// - `asset1_amount`: The amount of the first asset to be added to the pool.
		/// - `asset2_amount`: The amount of the second asset to be added to the pool.
		/// - `min_lp_token_amount`: The minimum amount of liquidity token that should be minted.
		/// - `deadline`: The deadline for the transaction to be executed.
		///
		/// **errors**
		/// - `CannotAddLiquidityWithSameAsset`: Cannot add liquidity with same asset.
		/// - `PoolNotFound`: Pool not found.
		/// - `DeadlinePassed`: Passed the deadline set for the transaction.
		/// - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
		/// - `NotEnoughLiquidityProvided`: Not enough liquidity provided.
		/// - `AmountMoreThanBalance`: Not enough balance.
		/// - `AddLiquidityFailed`: Add liquidity failed.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset1: T::AssetId,
			asset2: T::AssetId,
			asset1_amount: T::AssetBalance,
			asset2_amount: T::AssetBalance,
			min_lp_token_amount: T::AssetBalance,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let provider = ensure_signed(origin)?;

			// make sure the asset1 and asset2 are not the same.
			ensure!(asset1 != asset2, Error::<T>::CannotAddLiquidityWithSameAsset);

			// make sure the deadline has not passed.
			ensure!(Self::check_deadline(&deadline).is_ok(), Error::<T>::DeadlinePassed);

			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());

			let pool = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Add Liquidity >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let (asset1_actual_amount, asset2_actual_amount, lp_token_amount_minted) =
				Self::do_add_liquidity(
					&provider,
					asset1.clone(),
					asset2.clone(),
					asset1_amount,
					asset2_amount,
					min_lp_token_amount,
				)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::LiquidityAdded {
				liquidity_provider: provider.clone(),
				pool_id: Self::get_pool_id(asset1.clone(), asset2.clone()),
				asset1_amount_provided: asset1_actual_amount,
				asset2_amount_provided: asset2_actual_amount,
				lp_token: pool.lp_token.clone(),
				lp_token_amount_minted,
			});

			Ok(())
		}

		/// Remove liquidity from an existing liquidity pool.
		///
		/// **parameters**
		/// - `origin`: The account that is removing liquidity from the pool.
		/// - `asset1`: The first asset to be removed from the pool.
		/// - `asset2`: The second asset to be removed from the pool.
		/// - `asset1_min_receive_amount`: The minimum amount of the first asset that should be
		/// received.
		/// - `asset2_min_receive_amount`: The minimum amount of the second asset that should be
		/// received.
		/// - `lp_redeem_amount`: The amount of liquidity token that should be redeemed.
		/// - `deadline`: The deadline for the transaction to be executed.
		///
		/// **errors**
		/// - `PoolNotFound`: Pool not found.
		/// - `DeadlinePassed`: Passed the deadline set for the transaction.
		/// - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
		/// - `NotEnoughLiquidityToken`: Not enough liquidity token.
		/// - `RemoveLiquidityDidNotMeetMinimumAmount`: Remove liquidity did not meet minimum
		/// amount.
		/// - `AmountMoreThanBalance`: Not enough balance.
		/// - `EmptyPool`: Empty pool.
		/// - `CannotRedeemMoreThanTotalSupply`: Cannot redeem lp token more than its total supply
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset1: T::AssetId,
			asset2: T::AssetId,
			asset1_min_receive_amount: T::AssetBalance,
			asset2_min_receive_amount: T::AssetBalance,
			lp_redeem_amount: T::AssetBalance,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let remover = ensure_signed(origin)?;

			ensure!(Self::check_deadline(&deadline).is_ok(), Error::<T>::DeadlinePassed);

			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());

			let pool = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Remove Liquidity >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let (asset1_received_amount, asset2_received_amount) = Self::do_remove_liquidity(
				&remover,
				asset1.clone(),
				asset2.clone(),
				asset1_min_receive_amount,
				asset2_min_receive_amount,
				lp_redeem_amount,
			)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::LiquidityRemoved {
				remover,
				pool_id,
				asset1_received_amount,
				asset2_received_amount,
				lp_token: pool.lp_token.clone(),
				lp_token_burned: lp_redeem_amount,
			});

			Ok(())
		}

		/// Swap an exact amount of an asset in for as much of another asset as possible.
		///
		/// **parameters**
		/// - `origin`: The account that is swapping.
		/// - `asset_in`: The asset to be swapped in.
		/// - `asset_out`: The asset to be swapped out.
		/// - `exact_amount_in`: The exact amount of the asset that should be swapped in.
		/// - `min_amount_out`: The minimum amount of the asset that should be swapped out.
		/// - `deadline`: The deadline for the transaction to be executed.
		///
		/// **errors**
		/// - `DeadlinePassed`: Passed the deadline set for the transaction.
		/// - `CannotSwapZeroAmount`: Zero amount.
		/// - `CannotSwapSameAsset`: Cannot swap same asset.
		/// - `AmountMoreThanBalance`: Not enough balance.
		/// - `PoolNotFound`: Pool not found.
		/// - `EmptyPool`: Empty pool.
		/// - `InsufficientMinimumForSwap`: Insufficient minimum out for swap.
		/// - `AmountOutTooHigh`: Amount out too high.
		/// - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
		/// - `NotEnoughLiquidityToken`: Not enough liquidity token.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_exact_in_for_out(
			origin: OriginFor<T>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			exact_amount_in: T::AssetBalance,
			min_amount_out: T::AssetBalance,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let sender = ensure_signed(origin)?;

			// make sure the deadline has not passed.
			ensure!(Self::check_deadline(&deadline).is_ok(), Error::<T>::DeadlinePassed);

			// make sure the exact amount in and min amount out are not zero.
			ensure!(exact_amount_in > Zero::zero(), Error::<T>::CannotSwapZeroAmount);
			ensure!(min_amount_out > Zero::zero(), Error::<T>::CannotSwapZeroAmount);

			// make sure the asset in and asset out are not the same.
			ensure!(asset_in != asset_out, Error::<T>::CannotSwapSameAsset);

			//make sure sender has enough balance
			ensure!(
				<<T as Config>::AssetsRegistry>::reducible_balance(
					asset_in.clone(),
					&sender,
					Expendable,
					Polite,
				) >= exact_amount_in,
				Error::<T>::AmountMoreThanBalance
			);

			// get the pool id and pool account.
			let pool_id = Self::get_pool_id(asset_in.clone(), asset_out.clone());
			ensure!(Pools::<T>::contains_key(&pool_id), Error::<T>::PoolNotFound);
			let pool_account = Self::get_pool_account(&pool_id);

			// get the pool reserves.
			let asset_in_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset_in.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			let asset_out_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset_out.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			// make sure the pool is not empty.
			ensure!(
				asset_in_pool_reserve > Zero::zero() && asset_out_pool_reserve > Zero::zero(),
				Error::<T>::EmptyPool
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>> Calculate The Exact Amount Out >>>>>>>>>>>>>>>>>>>>>>>>>
			//
			// Formula:
			// exact_amount_out =
			// (exact_amount_in * (1000 - swap_fee) * asset_out_reserve) /      <---- Numerator
			// (asset_in_reserve * 1000 + exact_amount_in * (1000 - swap_fee))  <---- Denominator
			//
			// Note: both numerator and denominator are scaled by 1000 for precision on applying
			// fees.
			let amount_in_with_fee = exact_amount_in
				.checked_mul(&(1000u32 - T::SwapFee::get()).into())
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let numerator = amount_in_with_fee
				.checked_mul(&asset_out_pool_reserve)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let denominator = asset_in_pool_reserve
				.checked_mul(&1000u32.into())
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_add(&amount_in_with_fee)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let exact_amount_out =
				numerator.checked_div(&denominator).ok_or(Error::<T>::ArithmeticOverflow)?;

			// make sure the exact amount out is greater than or equal to the min amount out.
			ensure!(exact_amount_out >= min_amount_out, Error::<T>::InsufficientMinimumForSwap);

			// make sure the pool has enough reserve to swap.
			ensure!(exact_amount_out < asset_out_pool_reserve, Error::<T>::AmountOutTooHigh);

			// >>>>>>>>>>>>>>>>>>>>>>>>>> Do The Swap and Emit Event if Success >>>>>>>>>>>>>>>>>>>>
			// Event is emitted inside the do_swap function.
			Self::do_swap(
				sender.clone(),
				asset_in.clone(),
				asset_out.clone(),
				exact_amount_in,
				exact_amount_out,
			)?;

			Ok(())
		}

		/// Swap as little of an asset as possible for an exact amount of another asset.
		///
		/// **parameters**
		/// - `origin`: The account that is swapping.
		/// - `asset_in`: The asset to be swapped in.
		/// - `asset_out`: The asset to be swapped out.
		/// - `max_amount_in`: The maximum amount of the asset that should be swapped in.
		/// - `exact_amount_out`: The exact amount of the asset that should be swapped out.
		/// - `deadline`: The deadline for the transaction to be executed.
		///
		/// **errors**
		/// - `CannotSwapZeroAmount`: Cannot swap zero amount.
		/// - `CannotSwapSameAsset`: Cannot swap same asset.
		/// - `DeadlinePassed`: Passed the deadline set for the transaction.
		/// - `AmountMoreThanBalance`: Not enough balance.
		/// - `PoolNotFound`: Pool not found.
		/// - `EmptyPool`: Empty pool.
		/// - `AmountOutTooHigh`: Amount out too high.
		/// - `InsufficientMaximumForSwap`: Insufficient maximum in for swap.
		/// - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_in_for_exact_out(
			origin: OriginFor<T>,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			max_amount_in: T::AssetBalance,
			exact_amount_out: T::AssetBalance,
			deadline: BlockNumberFor<T>,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let sender = ensure_signed(origin)?;

			// make sure the deadline has not passed.
			ensure!(Self::check_deadline(&deadline).is_ok(), Error::<T>::DeadlinePassed);

			// make sure the max amount in and exact amount out are not zero.
			ensure!(max_amount_in > Zero::zero(), Error::<T>::CannotSwapZeroAmount);
			ensure!(exact_amount_out > Zero::zero(), Error::<T>::CannotSwapZeroAmount);

			// make sure the asset in and asset out are not the same.
			ensure!(asset_in != asset_out, Error::<T>::CannotSwapSameAsset);

			//make sure sender has enough balance
			ensure!(
				<<T as Config>::AssetsRegistry>::reducible_balance(
					asset_in.clone(),
					&sender,
					Expendable,
					Polite,
				) >= max_amount_in,
				Error::<T>::AmountMoreThanBalance
			);

			// get the pool id and pool account.
			let pool_id = Self::get_pool_id(asset_in.clone(), asset_out.clone());
			ensure!(Pools::<T>::contains_key(&pool_id), Error::<T>::PoolNotFound);
			let pool_account = Self::get_pool_account(&pool_id);

			// get the pool reserves.
			let asset_in_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset_in.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			let asset_out_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset_out.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			// make sure the pool is not empty.
			ensure!(
				asset_in_pool_reserve > Zero::zero() && asset_out_pool_reserve > Zero::zero(),
				Error::<T>::EmptyPool
			);

			// make sure the pool has enough reserve to swap.
			ensure!(exact_amount_out < asset_out_pool_reserve, Error::<T>::AmountOutTooHigh);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>> Calculate The Exact Amount In >>>>>>>>>>>>>>>>>>>>>>>>>>
			//
			// Formula:
			// exact_amount_in = 1 +
			// (asset_in_reserve * exact_amount_out * 1000) /               <---- Numerator
			// (asset_out_reserve - exact_amount_out) * (1000 - swap_fee))  <---- Denominator
			//
			// Note: both numerator and denominator are scaled by 1000 for precision on applying
			// fees.
			let numerator = asset_in_pool_reserve
				.checked_mul(&exact_amount_out)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_mul(&1000u32.into())
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let denominator = asset_out_pool_reserve
				.checked_sub(&exact_amount_out)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_mul(&(1000u32 - T::SwapFee::get()).into())
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let exact_amount_in = numerator
				.checked_div(&denominator)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_add(&One::one())
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			// make sure the exact amount in is less than or equal to the max amount in.
			ensure!(exact_amount_in <= max_amount_in, Error::<T>::InsufficientMaximumForSwap);

			// >>>>>>>>>>>>>>>>>>>>>>>>>> Do The Swap and Emit Event if Success >>>>>>>>>>>>>>>>>>>>
			// Event is emitted inside the do_swap function.
			Self::do_swap(
				sender.clone(),
				asset_in.clone(),
				asset_out.clone(),
				exact_amount_in,
				exact_amount_out,
			)?;

			Ok(())
		}

		/// Get the price of an asset in terms of another asset.
		/// The price is expressed as the amount of the unit asset that can be bought with one unit
		///
		/// **parameters**
		/// - `origin`: The account that is querying the price.
		/// - `asset`: The asset to be queried.
		/// - `price_unit`: The unit asset to be queried.
		/// - `asset_amount`: The amount of the asset to be queried.
		///
		/// **errors**
		/// - `PoolNotFound`: Pool not found.
		/// - `EmptyPool`: Empty pool.
		/// - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads(1).ref_time())]
		pub fn price_oracle(
			origin: OriginFor<T>,
			asset: T::AssetId,
			price_unit: T::AssetId,
			asset_amount: T::AssetBalance,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let querier = ensure_signed(origin)?;
			let pool_id = Self::get_pool_id(asset.clone(), price_unit.clone());
			ensure!(Pools::<T>::contains_key(&pool_id), Error::<T>::PoolNotFound);
			let pool_account = Self::get_pool_account(&pool_id);

			// get the pool reserves.
			let asset_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			let price_unit_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				price_unit.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			// ensure that the pool is not empty.
			ensure!(
				!asset_pool_reserve.is_zero() && !price_unit_pool_reserve.is_zero(),
				Error::<T>::EmptyPool
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Calculate Price >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			// marginal prices.
			//
			// Formula:
			//
			// marginal_price = asset_amount * asset_pool_reserve / price_unit_pool_reserve

			let marginal_price = asset_amount
				.checked_mul(&asset_pool_reserve)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_div(&price_unit_pool_reserve)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::PriceInfo {
				querier,
				pool_id,
				asset_amount,
				asset_pool_reserve,
				price_unit_pool_reserve,
				marginal_price,
			});

			Ok(())
		}

		/// Destroy an existing liquidity pool.
		///
		/// **parameters**
		/// - `origin`: The account that is destroying the pool.
		/// - `asset1`: The first asset in the pool.
		/// - `asset2`: The second asset in the pool.
		///
		/// **errors**
		/// - `PoolNotFound`: Pool not found.
		/// - `CannotDestroyPoolWithLiquidity`: Cannot destroy pool with liquidity.
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn destroy_pool(
			origin: OriginFor<T>,
			asset1: T::AssetId,
			asset2: T::AssetId,
		) -> DispatchResult {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let destroyer = ensure_signed(origin)?;

			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());
			ensure!(Pools::<T>::contains_key(&pool_id), Error::<T>::PoolNotFound);

			let pool_account = Self::get_pool_account(&pool_id);
			let pool = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			// make sure the pool is empty and the LP token total issuance is zero.
			ensure!(
				T::LpAssetsRegistry::reducible_balance(
					pool.lp_token.clone(),
					&pool_account,
					Expendable,
					Polite
				)
				.is_zero() && T::LpAssetsRegistry::total_issuance(pool.lp_token.clone()).is_zero(),
				Error::<T>::CannotDestroyPoolWithLiquidity
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Destroy LP Token >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			T::LpAssetsRegistry::start_destroy(pool.lp_token.clone(), Some(pool_account.clone()))?;
			T::LpAssetsRegistry::finish_destroy(pool.lp_token.clone())?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Update Storage >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			// Remove the pool from the storage.
			Pools::<T>::remove(pool_id.clone());

			// Transfer the pool deposit to the pool destroyer.
			T::NativeAsset::transfer(
				&pool_account,
				&destroyer,
				T::PoolSetupDeposit::get(),
				Expendable,
			)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::PoolDestroyed {
				destroyer,
				pool_id,
				pool_account,
				lp_token: pool.lp_token.clone(),
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Helper function to get the pool id from two asset ids. Pool id is a tuple of two sorted
		// asset ids.
		pub fn get_pool_id(asset1: T::AssetId, asset2: T::AssetId) -> PoolIdOf<T> {
			if asset1 <= asset2 {
				(asset1, asset2)
			} else {
				(asset2, asset1)
			}
		}

		// Helper function to get the pool account id from a pool id (Pool id is a tuple of two
		// sorted asset ids).
		pub fn get_pool_account(pool_id: &PoolIdOf<T>) -> T::AccountId {
			let encoded_pool_id = Hashable::blake2_256(&Encode::encode(&pool_id));

			Decode::decode(&mut TrailingZeroInput::new(encoded_pool_id.as_ref()))
				.expect("in our PBA exam, we assume all bytes can be turned into some account id")
		}

		// Helper function to check if the current block number is before the deadline block number.
		pub fn check_deadline(deadline: &BlockNumberFor<T>) -> Result<(), Error<T>> {
			ensure!(deadline >= &T::CurrentBlockNumber::get(), Error::DeadlinePassed);
			Ok(())
		}

		// Helper function to add liquidity to an existing liquidity pool.
		pub fn do_add_liquidity(
			provider: &T::AccountId,
			asset1: T::AssetId,
			asset2: T::AssetId,
			asset1_amount: T::AssetBalance,
			asset2_amount: T::AssetBalance,
			min_lp_token_amount: T::AssetBalance,
		) -> Result<(AssetBalanceOf<T>, AssetBalanceOf<T>, AssetBalanceOf<T>), DispatchError> {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());

			// make sure the amount of assets provided is greater than zero.
			ensure!(
				asset1_amount > Zero::zero() && asset2_amount > Zero::zero(),
				Error::<T>::InvalidLiquidityAmount
			);

			//make sure sender has enough balance
			ensure!(
				<<T as Config>::AssetsRegistry>::reducible_balance(
					asset1.clone(),
					&provider,
					Expendable,
					Polite,
				) >= asset1_amount,
				Error::<T>::AmountMoreThanBalance
			);

			//make sure sender has enough balance
			ensure!(
				<<T as Config>::AssetsRegistry>::reducible_balance(
					asset2.clone(),
					&provider,
					Expendable,
					Polite,
				) >= asset2_amount,
				Error::<T>::AmountMoreThanBalance
			);

			let pool = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			let pool_account = Self::get_pool_account(&pool_id);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Mint LP Tokens >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let lp_token_total_supply = T::LpAssetsRegistry::total_issuance(pool.lp_token.clone());

			// Get the current reserve balances of the pool
			let asset1_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset1.clone(),
				&pool_account,
				Expendable,
				Polite,
			);
			let asset2_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset2.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			// Calculate the actual amount of assets to be added to the pool
			let mut asset2_actual_amount = asset2_amount;
			let mut asset1_actual_amount = asset1_amount;

			if !asset1_pool_reserve.is_zero() && !asset2_pool_reserve.is_zero() {
				let asset2_amount_temp = asset1_amount
					.checked_mul(&asset2_pool_reserve)
					.ok_or(Error::<T>::ArithmeticOverflow)?
					.checked_div(&asset1_pool_reserve)
					.ok_or(Error::<T>::ArithmeticOverflow)?;

				if asset2_amount_temp <= asset2_amount {
					asset2_actual_amount = asset2_amount_temp;
				} else {
					let asset1_amount_temp = asset2_amount
						.checked_mul(&asset1_pool_reserve)
						.ok_or(Error::<T>::ArithmeticOverflow)?
						.checked_div(&asset2_pool_reserve)
						.ok_or(Error::<T>::ArithmeticOverflow)?;

					asset1_actual_amount = asset1_amount_temp;
				}
			}

			// Calculate the amount of LP tokens to mint
			//
			// When the pool is empty:
			// lp_token_mint_amount = sqrt(asset1_amount * asset2_amount)
			//
			// When the pool is not empty:
			// lp_token_mint_amount = min(
			//     asset1_amount * lp_token_total_supply / asset1_pool_reserve,
			//     asset2_amount * lp_token_total_supply / asset2_pool_reserve,
			// )
			let lp_token_mint_amount: T::AssetBalance = {
				if lp_token_total_supply.is_zero() {
					asset1_amount
						.checked_mul(&asset2_amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?
						.integer_sqrt()
				} else {
					Ord::min(
						asset1_amount
							.checked_mul(&lp_token_total_supply)
							.ok_or(Error::<T>::ArithmeticOverflow)?
							.checked_div(&asset1_pool_reserve)
							.ok_or(Error::<T>::ArithmeticOverflow)?,
						asset2_amount
							.checked_mul(&lp_token_total_supply)
							.ok_or(Error::<T>::ArithmeticOverflow)?
							.checked_div(&asset2_pool_reserve)
							.ok_or(Error::<T>::ArithmeticOverflow)?,
					)
				}
			};

			// make sure the amount of LP tokens minted is greater than the minimum amount specified
			ensure!(
				lp_token_mint_amount >= min_lp_token_amount,
				Error::<T>::NotEnoughLiquidityProvided
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Transfer Assets to Pool >>>>>>>>>>>>>>>>>>>>>>>>>>>>
			T::AssetsRegistry::transfer(
				asset1.clone(),
				&provider,
				&pool_account,
				asset1_actual_amount,
				Expendable,
			)?;
			T::AssetsRegistry::transfer(
				asset2.clone(),
				&provider,
				&pool_account,
				asset2_actual_amount,
				Expendable,
			)?;

			// Mint the LP tokens to the liquidity provider
			T::LpAssetsRegistry::mint_into(pool.lp_token.clone(), &provider, lp_token_mint_amount)?;

			Ok((asset1_actual_amount, asset2_actual_amount, lp_token_mint_amount))
		}

		// Helper function to remove liquidity from an existing liquidity pool.
		pub fn do_remove_liquidity(
			remover: &T::AccountId,
			asset1: T::AssetId,
			asset2: T::AssetId,
			asset1_min_receive_amount: T::AssetBalance,
			asset2_min_receive_amount: T::AssetBalance,
			lp_redeem_amount: T::AssetBalance,
		) -> Result<(AssetBalanceOf<T>, AssetBalanceOf<T>), DispatchError> {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation & Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let pool_id = Self::get_pool_id(asset1.clone(), asset2.clone());

			// make sure the amount of LP tokens to redeem is greater than zero.
			ensure!(lp_redeem_amount > Zero::zero(), Error::<T>::NotEnoughLiquidityToken);

			let pool = Pools::<T>::get(pool_id.clone()).ok_or(Error::<T>::PoolNotFound)?;

			let pool_account = Self::get_pool_account(&pool_id);

			//make sure sender has enough balance
			ensure!(
				<<T as Config>::LpAssetsRegistry>::reducible_balance(
					pool.lp_token.clone(),
					&remover,
					Expendable,
					Polite,
				) >= lp_redeem_amount,
				Error::<T>::AmountMoreThanBalance
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Calculate Asset Amount >>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			let lp_token_total_supply = T::LpAssetsRegistry::total_issuance(pool.lp_token.clone());
			ensure!(
				lp_token_total_supply >= lp_redeem_amount,
				Error::<T>::CannotRedeemMoreThanTotalSupply
			);

			// Get the current reserve balances of the pool
			let asset1_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset1.clone(),
				&pool_account,
				Expendable,
				Polite,
			);
			let asset2_pool_reserve = <<T as Config>::AssetsRegistry>::reducible_balance(
				asset2.clone(),
				&pool_account,
				Expendable,
				Polite,
			);

			// Calculate the amount of asset1 and asset2 to receive
			//
			// Formula:
			// asset1_receive_amount =
			// lp_redeem_amount * asset1_pool_reserve / lp_token_total_supply
			//
			// asset2_receive_amount =
			// lp_redeem_amount * asset2_pool_reserve / lp_token_total_supply
			let asset1_receive_amount = lp_redeem_amount
				.checked_mul(&asset1_pool_reserve)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_div(&lp_token_total_supply)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			let asset2_receive_amount = lp_redeem_amount
				.checked_mul(&asset2_pool_reserve)
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.checked_div(&lp_token_total_supply)
				.ok_or(Error::<T>::ArithmeticOverflow)?;

			// making sure the amount of asset1 and asset2 received is greater than the minimum
			// amount requested
			ensure!(
				asset1_receive_amount >= asset1_min_receive_amount,
				Error::<T>::RemoveLiquidityDidNotMeetMinimumAmount
			);

			ensure!(
				asset2_receive_amount >= asset2_min_receive_amount,
				Error::<T>::RemoveLiquidityDidNotMeetMinimumAmount
			);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Burn & Transfer Assets >>>>>>>>>>>>>>>>>>>>>>>>>>>>>

			// Burn the LP tokens from the remover
			T::LpAssetsRegistry::burn_from(
				pool.lp_token.clone(),
				&remover,
				lp_redeem_amount,
				Exact,
				Polite,
			)?;

			// Transfer the assets to the remover
			T::AssetsRegistry::transfer(
				asset1.clone(),
				&pool_account,
				&remover,
				asset1_receive_amount,
				Expendable,
			)?;

			T::AssetsRegistry::transfer(
				asset2.clone(),
				&pool_account,
				&remover,
				asset2_receive_amount,
				Expendable,
			)?;

			Ok((asset1_receive_amount, asset2_receive_amount))
		}

		// Helper function to swap an exact amount of an asset in for an exact amount of another
		// asset.
		// This function assumes all the proper validation has been done.
		pub fn do_swap(
			sender: T::AccountId,
			asset_in: T::AssetId,
			asset_out: T::AssetId,
			exact_amount_in: T::AssetBalance,
			exact_amount_out: T::AssetBalance,
		) -> Result<(), DispatchError> {
			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Setup >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			let pool_id = Self::get_pool_id(asset_in.clone(), asset_out.clone());
			let pool_account = Self::get_pool_account(&pool_id);

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Transfer Assets >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			T::AssetsRegistry::transfer(
				asset_in.clone(),
				&sender,
				&pool_account,
				exact_amount_in,
				Expendable,
			)?;

			T::AssetsRegistry::transfer(
				asset_out.clone(),
				&pool_account,
				&sender,
				exact_amount_out,
				Expendable,
			)?;

			// >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Emit Event >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
			Self::deposit_event(Event::SwapSucceeded {
				user: sender,
				asset_in,
				asset_out,
				amount_in: exact_amount_in,
				amount_out: exact_amount_out,
			});

			Ok(())
		}

		// Mint an amount of an asset to an account (root only).
		pub fn mint_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			amount: T::AssetBalance,
			receiver: T::AccountId,
		) -> DispatchResult {
			// make sure the caller is root.
			ensure_root(origin)?;

			// make sure the amount is not zero.
			ensure!(amount > Zero::zero(), Error::<T>::CannotMintZeroAmount);

			// mint the asset to the receiver.
			T::AssetsRegistry::mint_into(asset_id, &receiver, amount)?;
			Ok(())
		}
	}
}

// Look at `../interface/` to better understand this API.
impl<T: Config> pba_interface::DexInterface for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = T::AssetId;
	type AssetBalance = T::AssetBalance;

	fn setup_account(_who: Self::AccountId) -> DispatchResult {
		unimplemented!()
	}

	fn mint_asset(
		_who: Self::AccountId,
		_token_id: Self::AssetId,
		_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn asset_balance(_who: Self::AccountId, _token_id: Self::AssetId) -> Self::AssetBalance {
		unimplemented!()
	}

	fn swap_fee() -> u16 {
		unimplemented!()
	}

	fn lp_id(_asset_a: Self::AssetId, _asset_b: Self::AssetId) -> Self::AssetId {
		unimplemented!()
	}

	fn add_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_amount_a: Self::AssetBalance,
		_amount_b: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_token_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_exact_in_for_out(
		_who: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_exact_in: Self::AssetBalance,
		_min_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_in_for_exact_out(
		_origin: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_max_in: Self::AssetBalance,
		_exact_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}
}
