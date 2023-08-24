# Seacliff Program 🌊📜

Seacliff is a pioneering Solana program that brings the concept of dominant assurance contracts to the decentralized world of blockchain. Crafted meticulously with the Anchor framework, it offers a unique set of functionalities tailored to establish, manage, and operate assurance contracts. ***More details about this project can be found in this [project proposal](https://docs.google.com/document/d/1RX5b0FcILSrzgP73OImFT2dlxnbb_98j5a1_6TvCQ58/edit).***

## Table of Contents 📚

- [**Features**](#features-🌟)
- [**Setup**](#setup-🛠)
- [**Technical Details**](#technical-details-📦)
    - [**Instructions**](#instructions)
    - [**State**](#state)


## Features 🌟

- ✅ **Contract Initialization**: Empowers users to initiate contracts with detailed parameters, including the goal, lifespan, and refund bonuses.
- ✅ **Dynamic Error Handling**: Provides precise error messages for various scenarios ensuring a smooth user experience.
- ✅ **Pledging Mechanism**: Allows backers to pledge to contracts seamlessly.
- ✅ **NFT Minting**: pledger NFTs are minted and distributed to backers based on their pledge amounts.
- ✅ **Bonus Redemption**: Facilitates the process of redeeming funds based on contract conditions and outcomes.
- ✅ **NFT Burning**: pledger NFTs are burned upon contract completion or bonus redemption, ensuring a fair and transparent process.

## Setup 🛠

### Install Dependencies

- Go [here](https://www.rust-lang.org/tools/install) to install **Rust**.

- Go [here](https://docs.solana.com/cli/install-solana-cli-tools) to install **Solana** and then run `solana-keygen new` to create a keypair at the default location. Anchor uses this keypair to run your program tests.

- Go [here](https://yarnpkg.com/getting-started/install) to install **Yarn**.

- Go [here](https://www.anchor-lang.com/docs/installation) to install **Anchor**.

### Build

```
anchor build
```

Builds programs in the workspace targeting Solana's BPF runtime and emitting IDLs in the target/idl directory.

```
anchor build --verifiable
```

Runs the build inside a docker image so that the output binary is deterministic (assuming a Cargo.lock file is used). This command must be run from within a single crate subdirectory within the workspace. For example, programs/<my-program>/.

### Cluster

```
anchor cluster list
```

This lists **cluster endpoints**:

- Mainnet - <https://api.mainnet-beta.solana.com>
- Devnet  - <https://api.devnet.solana.com>
- Testnet - <https://api.testnet.solana.com>

### Deploy

```
anchor deploy
```

Deploys all programs in the workspace to the configured cluster.

### Test

```
anchor test
```

Run an integration test suit against the configured cluster, deploying new versions of all workspace programs before running them. If the configured network is a localnet, then automatically starts the localnetwork and runs the test.

### More info on Anchor CLI [here](https://www.anchor-lang.com/docs/cli)

## Technical Details 📦
### Instructions

#### `create_contract`
Initializes the contract and transfers the refund bonus from the proposer to the contract.

**Requirements**
- The contract must be uninitialized.
- The proposer must have enough lamports to cover the refund bonus.
- The goal must be greater than or equal to the minimum goal.
- The refund bonus must be greater than or equal to the minimum refund bonus.
- The goal must be greater than the refund bonus.
- The lifespan must be greater than or equal to the minimum lifespan.
- The lifespan must be less than or equal to the maximum lifespan.

**Parameters**
- `proposer` - The proposer of the contract.
- `goal` - The total amount of lamports the contract wishes to raise before the lifespan ends.
- `lifespan` - The lifespan during which the contract must reach its goal.
- `refund_bonus` - The amount of lamports to be paid to the backers if the contract fails to reach its goal before the lifespan ends.
- `system_program` - The system program.
- `pda_account` - The PDA account of the DAC program.
- `nft_mint` - The mint of the NFT that is minted to the backer when they pledge to the contract.

**Errors**
- `ContractAlreadyInitialized` - Cannot initialize an already initialized contract.
- `GoalLessThanMinimum` - Cannot initialize a contract with a goal less than the minimum goal.
- `RefundBonusLessThanMinimum` - Cannot initialize a contract with a refund bonus less than the minimum refund bonus.
- `GoalLessThanRefundBonus` - Cannot initialize a contract with a goal less than the refund bonus.
- `LifespanLessThanMinimum` - Cannot initialize a contract with a lifespan less than the minimum lifespan.
- `LifespanLGreaterThanMaximum` - Cannot initialize a contract with a lifespan greater than the maximum lifespan.
- `NotEnoughLamportsForRefundBonus` - The proposer does not have enough lamports to cover the refund bonus.
- `ArithmeticOverflow` - The operation would cause an overflow.
---

### `pledge`
 Allows backers to pledge to a contract.
    Makes a pledge to the contract and transfers the pledge from the backer to the contract.

**Requirements**
- The contract must be active.
- The contract must not have passed its deadline.
- The amount must be greater than or equal to the minimum pledge.
- The backer must have enough lamports to cover the pledge.

**Parameters**
- `backer` - The backer of the contract.
- `amount` - The amount of lamports to pledge.
- `system_program` - The system program.
- `pda_account` - The PDA account of the DAC program.

**Errors**
- `ContractNotActive` - Cannot pledge to a contract that is not active.
- `DeadlinePassed` - Cannot pledge to a contract that has passed its deadline.
- `PledgeLessThanMinimum` - Cannot pledge less than the minimum pledge.
- `NotEnoughLamportsForPledge` - The backer does not have enough lamports to cover the pledge.
- `ArithmeticOverflow` - The operation would cause an overflow.

---
### `close_contract` 
Closes a contract and burns the pledger NFTs.

  Closes the contract.
      Transfers the refund bonus and all pledged funds to the proposer if the contract reached its goal.
      Otherwise, wait for the backers to redeem their refund bonus.
     
**Requirements**
- The contract must be active.
- The contract must have passed its deadline.

**Parameters**
- `proposer` - The proposer of the contract.
- `system_program` - The system program.
- `pda_account` - The PDA account of the DAC program.

**Errors**
- `ContractNotActive` - Cannot close a contract that is not active.
- `CloseBeforeDeadline` - Cannot close a contract that has not passed its deadline.
- `ArithmeticOverflow` - The operation would cause an overflow.
---

### `redeem_refund_bonus`
Allows backers to redeem their bonus based on the contract outcomes, and NFT metadata.
**More Documentations Coming**

---
### States
### `DominantAssuranceContract`
    
#### Fields:
- `proposer: Pubkey`: The proposer of the contract, 32 bytes.

- `goal: u64`: The total amount of lamports the contract wishes to raise before the lifespan ends, 8 bytes.
    
- `birth: u64`: The time at which the contract was created, 8 bytes.

- `lifespan: u64`: The lifespan during which the contract must reach its goal, 8 bytes.

- `refund_bonus: u64`: The amount of lamports to be paid to the backers if the contract fails to reach its goal before the lifespan ends. The refund bonus is deposited by the contract proposer when the contract is created, 8 bytes.

- `total_pledged: u64`: The total amount of lamports the contract has received from backers, 8 bytes.

- `total_backers: u64`: The total number of backers to the contract.

- `state: ContractState`: The state of the contract, 2 bytes. More details below.

- `nft_mint: Pubkey`: The mint account of the backer NFT. The NFT is burned when the backer redeems their refund bonus or when closing a funded contract. The NFT mint authority is the PDA account of the DAC program, 32 bytes.

#### Constants:
- `MAXIMUM_SIZE: usize`: The maximum size of the contract account, currently 114 bytes.

- `MINIMUM_GOAL: u64`: The minimum goal of a contract, currently 10 SOL.

- `MINIMUM_REFUND_BONUS: u64`: The minimum refund bonus of a contract, currently 1 SOL.

- `MINIMUM_LIFESPAN: u64`: The minimum lifespan of a contract, currently 1 day.

- `MAXIMUM_LIFESPAN: u64`: The maximum lifespan of a contract, currently 1 year.

- `MINIMUM_PLEDGE: u64`: The minimum pledge of a contract, currently 0.1 SOL.


### `ContractState`

#### Variants:
- `Uninitialized`: The contract is uninitialized.
- `Active`: The contract is active.
- `Closed {None}`: The contract did not reach its goal and is closed, backers can redeem their refund bonus when the contract is in this state.
- `Closed {Some()}`: The contract is successfully funded.

### More info check out the instruction [here](https://www.anchor-lang.com/docs/cli#idl) to get the Interface Definition Language (IDL) of this program
