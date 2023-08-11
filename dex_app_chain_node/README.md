# Decentralized Exchange Node


🚀 Build with FRAME & [Substrate](https:www.substrate.io/) 🚀

## Background

This project aims to develop a unique blockchain node using an array of customized FRAME pallets. It is built on the foundation of Substrate and employs an automated liquidity protocol inspired by [Uniswap V2's constant product Formula:](https:docs.uniswap.org/contracts/v2/concepts/protocol-overview/glossary#constant-product-Formula:). The project is designed to provide a seamless trading experience for users and to maximize storage efficiency which is crucial for long term usability. The project is built with following attributes:

✅ **Liquidity Pool Creation:** The project provides an opportunity for users to create a liquidity pool (exchange) on any pair of tokens. The pool creator is required to deposit an equal value of both tokens in the pool, and lock up 100 native tokens as a storage deposit, which is returned when the pool is closed.

✅ **Liquidity Addition/Removal:** The system allows users to add/remove liquidity to/from an existing exchange, thereby enhancing the liquidity of the exchange and improving its overall functionality.

✅ **Token Swapping:** The project enables users to effortlessly swap tokens with different configurations. **However this project currently only support single swaps, multi-hop swaps will be introduced in future development.**

✅ **Price Oracle:** The system allows users to query token prices, providing them with up-to-date market information to facilitate informed decision-making.

✅ **Incentivization for Liquidity Provider:** Liquidity providers can earn 0.3% exchange fee from every swap in the pool in the form of liquidity provider (LP) tokens. This feature creates an incentive for users to provide liquidity.

✅ **Incentivization for Storage Cleanup:** Using storage deposits as an incentive to clean up unused storage, the project rewards users for cleaning up unused liquidity pools. .

✅ **Deadline for Transactions:** The system allows users to set a deadline for their transactions, thereby reducing the risk of front-running.

## Technical Details

### [Here are some diagrams and detailed descriptions to help you understand how the system works](https://docs.uniswap.org/contracts/v2/concepts/protocol-overview/how-uniswap-works)

### Types

* `RuntimeEvent`: The overarching event type. Because this pallet emits events, it depends on the runtime's definition of an event. See the [FRAME Events Documentation](https:docs.substrate.io/build/events-and-Errors:/) for details.

* `NativeAsset`: The native asset type. This is the asset that is used to pay for pool setup storage deposits. **This asset currently cannot be swapped with other assets, but swapping with this asset is planned for future development.**

* `NativeBalance`: The balance type of the native asset.

* `AssetBalance`: The balance type of the non-native assets. This is a generic type that can be used to represent any asset balance. Any asset of this type can be used to create pool and be swapped with any other asset of this type.

* `AssetId`: The non-native asset identifier type. This is a generic type that can be used to represent any non-native asset. This type is also used to construct liquidity pool identifiers and liquidity pool account identifiers.

* `LpTokenId`: The liquidity pool token identifier type. This is a generic type that can be used to represent any liquidity pool token. This type is part of the PoolInfo struct in the Pool storage item.

* `AssetsRegistry`: The asset registry type. This is a generic type that can be used to keep track of all supported non-native assets in the decentralized exchange.

* `LpAssetsRegistry`: The liquidity pool token registry type. This is a generic type that can be used to keep track of all supported liquidity pool tokens in the decentralized exchange.

* `CurrentBlockNumber`: The current block number type. This is a generic type that can be used to represent the current block number. **This is used to set deadlines for transactions, reducing the risk of front-running**

### Constants

* `SwapFee`: The swap fee charged by the decentralized exchange. This is a percentage value represented as a rational number. The default value is 0.003, which is equivalent to 0.3%. This acts as an incentive for liquidity providers to provide liquidity to the decentralized exchange.

* `PoolSetupDeposit`: The amount of storage deposit in native asset required to create a liquidity pool. This is a fixed value defaults to 100. This is used to prevent spamming of the blockchain by requiring users to pay a storage deposit to create a liquidity pool. It also acts as an incentive for users to clean up unused liquidity pools.


### Storage Items

* `Pools`: The storage item that keeps track of all liquidity pools in the decentralized exchange. This is a map from a pool identifier to a pool information struct which contains the LP token identifier of the corresponding pool.

* `NextLpTokenId`: The storage item that keeps track of the next available liquidity pool token identifier. This is a simple counter that is incremented every time a new liquidity pool token is created.

### Extrinsics

 * `create_pool`:  Create a new liquidity pool and provide initial liquidity. The creator will need to make a pool setup deposit in native token to incentivize cleaning up unused pools. 
    <details>
 
   **Parameters:**

    - `origin`: The account that is creating the pool.

    - `asset1`: The first asset to be added to the pool.

    - `asset2`: The second asset to be added to the pool.

    - `amount1`: The amount of the first asset to be added to the pool.

    - `amount2`: The amount of the second asset to be added to the pool.

    - `min_lp_token_amount`: The minimum amount of liquidity token that should be minted.
   
    **Errors:**

    - `CannotCreatePoolWithSameAsset`: Cannot create pool with same asset.

    - `PoolAlreadyExists`: Pool already exists.

    - `IncorrectLpTokenId`: Incorrect LP token id.

    - `InvalidLiquidityAmount`: Invalid liquidity amount for an asset.

    - `NotEnoughToPayForPoolSetupDeposit`: Sender does not have enough native asset balance

    - `AmountMoreThanBalance`: Not enough balance.

    - `AddLiquidityFailed`: Add liquidity failed.
     
---

* `add_liquidity`:   Add liquidity to an existing liquidity pool.
   <details>

    **Parameters:**

    - `origin`: The account that is adding liquidity to the pool.
    - `asset1`: The first asset to be added to the pool.
    - `asset2`: The second asset to be added to the pool.
    - `asset1_amount`: The amount of the first asset to be added to the pool.
    - `asset2_amount`: The amount of the second asset to be added to the pool.
    - `min_lp_token_amount`: The minimum amount of liquidity token that should be minted.
    - `deadline`: The deadline for the transaction to be executed.

    **Formula:**

    ```rust
    /// When the pool is empty:
   /// lp_token_mint_amount = sqrt(asset1_amount * asset2_amount)
   ///
   /// When the pool is not empty:
   /// lp_token_mint_amount = min(
   ///     asset1_amount * lp_token_total_supply / asset1_pool_reserve,
   ///     asset2_amount * lp_token_total_supply / asset2_pool_reserve,
   /// )
    ```
   
    **Errors:**
    - `CannotAddLiquidityWithSameAsset`: Cannot add liquidity with same asset.
    - `PoolNotFound`: Pool not found.
    - `DeadlinePassed`: Passed the deadline set for the transaction.
    - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
    - `NotEnoughLiquidityProvided`: Not enough liquidity provided.
    - `AmountMoreThanBalance`: Not enough balance.
    - `AddLiquidityFailed`: Add liquidity failed.
     
---

* `remove_liquidity`:   Remove liquidity from an existing liquidity pool.
   <details>

    **Parameters:**

    - `origin`: The account that is removing liquidity from the pool.
    - `asset1`: The first asset to be removed from the pool.
    - `asset2`: The second asset to be removed from the pool.
    - `asset1_min_receive_amount`: The minimum amount of the first asset that should be
    received.
    - `asset2_min_receive_amount`: The minimum amount of the second asset that should be
    received.
    - `lp_redeem_amount`: The amount of liquidity token that should be redeemed.
    - `deadline`: The deadline for the transaction to be executed.

    **Formula:**

    ```rust
    // asset1_receive_amount =
   // lp_redeem_amount * asset1_pool_reserve / lp_token_total_supply
   //
   // asset2_receive_amount =
   // lp_redeem_amount * asset2_pool_reserve / lp_token_total_supply
    ```
   
    **Errors:**

    - `PoolNotFound`: Pool not found.
    - `DeadlinePassed`: Passed the deadline set for the transaction.
    - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
    - `NotEnoughLiquidityToken`: Not enough liquidity token.
    - `RemoveLiquidityDidNotMeetMinimumAmount`: Remove liquidity did not meet minimum
    amount.
    - `AmountMoreThanBalance`: Not enough balance.
    - `EmptyPool`: Empty pool.
    - `CannotRedeemMoreThanTotalSupply`: Cannot redeem lp token more than its total supply
     
---
* `swap_exact_in_for_out`:   Swap an exact amount of an asset in for as much of another asset as possible.
   <details>

    **Parameters:**

    - `origin`: The account that is swapping.
    - `asset_in`: The asset to be swapped in.
    - `asset_out`: The asset to be swapped out.
    - `exact_amount_in`: The exact amount of the asset that should be swapped in.
    - `min_amount_out`: The minimum amount of the asset that should be swapped out.
    - `deadline`: The deadline for the transaction to be executed.

    **Formula:**
    ```rust
    /// exact_amount_out =
    /// (exact_amount_in *(1000 - swap_fee)* asset_out_reserve) /      <---- Numerator
    /// (asset_in_reserve *1000 + exact_amount_in* (1000 - swap_fee))  <---- Denominator
    /// 
    /// Note: both numerator and denominator are scaled by 1000 for precision on applying fees.
    ```
   
    **Errors:**
    - `DeadlinePassed`: Passed the deadline set for the transaction.
    - `CannotSwapZeroAmount`: Zero amount.
    - `CannotSwapSameAsset`: Cannot swap same asset.
    - `AmountMoreThanBalance`: Not enough balance.
    - `PoolNotFound`: Pool not found.
    - `EmptyPool`: Empty pool.
    - `InsufficientMinimumForSwap`: Insufficient minimum out for swap.
    - `AmountOutTooHigh`: Amount out too high.
    - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
    - `NotEnoughLiquidityToken`: Not enough liquidity token.

---

* `swap_in_for_exact_out`:   Swap as little of an asset as possible for an exact amount of another asset.
    <details>

    **Parameters:**

    - `origin`: The account that is swapping.
    - `asset_in`: The asset to be swapped in.
    - `asset_out`: The asset to be swapped out.
    - `max_amount_in`: The maximum amount of the asset that should be swapped in.
    - `exact_amount_out`: The exact amount of the asset that should be swapped out.
    - `deadline`: The deadline for the transaction to be executed.

    **Formula:**

    ```rust
   /// exact_amount_in = 1 +
   /// (asset_in_reserve * exact_amount_out * 1000) /               <---- Numerator
   /// (asset_out_reserve - exact_amount_out) * (1000 - swap_fee))  <---- Denominator
   ///
   /// Note: both numerator and denominator are scaled by 1000 for precision on applying
   /// fees.
    ```
   
    **Errors:**

    - `CannotSwapZeroAmount`: Cannot swap zero amount.
    - `CannotSwapSameAsset`: Cannot swap same asset.
    - `DeadlinePassed`: Passed the deadline set for the transaction.
    - `AmountMoreThanBalance`: Not enough balance.
    - `PoolNotFound`: Pool not found.
    - `EmptyPool`: Empty pool.
    - `AmountOutTooHigh`: Amount out too high.
    - `InsufficientMaximumForSwap`: Insufficient maximum in for swap.
    - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
     
---

* `price_oracle`:   Get the price of an asset in terms of another asset.
    The price is expressed as the amount of the unit asset that can be bought with one unit
    
    <details>

    **Parameters:**
    - `origin`: The account that is querying the price.
    - `asset`: The asset to be queried.
    - `price_unit`: The unit asset to be queried.
    - `asset_amount`: The amount of the asset to be queried.

       **Formula:**

    ```rust
   /// marginal_price = asset_amount * asset_pool_reserve / price_unit_pool_reserve
    ```
   
    **Errors:**
    - `PoolNotFound`: Pool not found.
    - `EmptyPool`: Empty pool.
    - `ArithmeticOverflow`: Overflow when doing arithmetic operations.
     
---

* `destroy_pool`:   Destroy an existing liquidity pool. The pool destroyer will be refunded the storage deposit in native token. (**NOTE‼️ Currently it is not possible to destroy a pool with liquidity. Even with the logic in this function. Because of the constant product formula, it is not possible to remove all liquidity from a pool. This is a known issue and will be fixed in future development.** )
   
    <details>

    **Parameters:**
    - `origin`: The account that is destroying the pool.
    - `asset1`: The first asset in the pool.
    - `asset2`: The second asset in the pool.
   
    **Errors:**
    - `PoolNotFound`: Pool not found.
    - `CannotDestroyPoolWithLiquidity`: Cannot destroy pool with liquidity.

---

## Trade Offs

In every project, striking a balance between different aspects is essential to optimize its performance. In this project, the primary goal is to minimize storage consumption, which might lead to a compromise in execution time for the sake of storage efficiency.

The storage items are designed to be as compact as possible. For example, the `Pools` storage item is a map from a pool identifier to a pool information struct which contains the LP token identifier of the corresponding pool. This design is chosen to minimize storage consumption. However, this design requires an additional lookup to get the other information of the pool. This might lead to a compromise in execution time.

The accounts are defaulted to be able to go below the existential deposit, which will cause the account to be killed. This is done to minimize storage consumption. However, this might lead to a compromise in security.

The decision to prioritize storage efficiency over weight efficiency was strategic, intending to improve the long-term usability of the project. The choice leans towards maintaining a leaner, more compact storage structure, which is crucial for sustainable project operation in the long run.
     

## Future Roadmap
**Here are some ideas of where the future improvements can be made:**

 - **Concentrated Liquidity:** Concentrated liquidity feature, which will allow liquidity providers to allocate liquidity to desired price ranges. This mechanism will not only make liquidity provision more capital efficient, but will also reduce liquidity providers' impermanent loss.

- **Governance Mechanism:** Governance mechanism using native assets. This system will empower the community to make major decisions on swapping fee, storage deposit and become self sustaining.

- **Native Asset Swappability:** Make the native asset swappable. This will provide an additional layer of utility to the native asset, promoting its use within the ecosystem. Users will be able to directly swap the native asset for other tokens, thereby fostering a more seamless and efficient trading experience.

- **Rectification of the 'Destroy_Pool' Function:** Rectify the 'destroy_pool' function. This will ensure the effective removal of unused liquidity pools, optimizing the efficiency of the entire system. 

- **Multi-Hop Swaps:** Introducing multi-hop swaps. This feature will allow users to swap tokens even when a direct pool does not exist between the two desired tokens. Instead, the system will find a route through multiple pools, allowing users to make the swap. This will greatly increase the number of possible trades and enhance the flexibility of the platform.


## Setup

Please first check the latest information on getting starting with Substrate dependencies required to build this project [here](https:docs.substrate.io/main-docs/install/).

### Development Testing

To test while developing, without a full build (thus reduce time to results):

```sh
cargo t -p pallet-dex
cargo t -p pallet-dpos
cargo t -p pallet-voting
cargo t -p <other crates>
```

### Build

Build the node without launching it, with `release` optimizations:

```sh
cargo b -r
```

### Run

Build and launch the node, with `release` optimizations:

```sh
cargo r -r -- --dev
```

### CLI Docs

Once the project has been built, the following command can be used to explore all CLI arguments and subcommands:

```sh
./target/release/node-template -h
```
