use crate::errors::ContractError;
use anchor_lang::{prelude::*, system_program};
use num_traits::*;

#[account]
pub struct DominantAssuranceContract {
    /// The proposer of the contract.
    ///
    /// 32 bytes
    proposer: Pubkey,
    /// The total amount of lamports the contract wishes to raise before the lifespan ends.
    ///
    /// 8 bytes
    goal: u64,
    /// The time at which the contract was created.
    /// This field is represented in UnixTimestamp.
    ///
    /// 8 bytes
    birth: u64,
    /// The lifespan during which the contract must reach its goal.
    /// This field is represented in UnixTimestamp.
    ///
    /// 8 bytes
    lifespan: u64,
    /// The amount of lamports to be paid to the backers if the contract fails to reach its goal before the lifespan ends.
    /// The refund bonus is deposited by the contract proposer when the contract is created.
    ///
    /// 8 bytes
    refund_bonus: u64,
    /// The total amount of lamports the contract has received from backers.
    ///
    /// 8 bytes
    total_pledged: u64,
    /// The total number of backers to the contract.
    ///
    /// 8 bytes
    total_backers: u64,
    /// The state of the contract.
    /// The contract is uninitialized if it has not been initialized yet.
    /// The contract is active if the lifespan has not ended.
    /// The contract is closed if the lifespan has ended.
    ///
    /// 2 bytes
    state: ContractState,
    // TODO: Might need more fields here.
    /// The mint of the NFT that is minted to the backer when they pledge to the contract.
    /// The NFT is burned when the backer redeems their refund bonus.
    /// The NFT mint authority is the PDA account of the DAC program.
    ///
    /// 32 bytes
    nft_mint: Pubkey,
}

impl DominantAssuranceContract {
    pub const MAXIMUM_SIZE: usize = 32 + 8 + 8 + 8 + 8 + 8 + 8 + 2 + 32; // TODO: Might need to be changed if new fields are added.
    pub const MINIMUM_GOAL: u64 = 10000000000u64; // 10 SOL
    pub const MINIMUM_REFUND_BONUS: u64 = 1000000000u64; // 1 SOL
    pub const MINIMUM_LIFESPAN: u64 = 24 * 60 * 60; // 1 day
    pub const MAXIMUM_LIFESPAN: u64 = 365 * 24 * 60 * 60; // approx. 1 years
    pub const MINIMUM_PLEDGE: u64 = 100000000u64; // 0.1 SOL

    /// Initializes the contract and transfers the refund bonus from the proposer to the contract.
    ///
    /// **Requirements**
    /// - The contract must be uninitialized.
    /// - The proposer must have enough lamports to cover the refund bonus.
    /// - The goal must be greater than or equal to the minimum goal.
    /// - The refund bonus must be greater than or equal to the minimum refund bonus.
    /// - The goal must be greater than the refund bonus.
    /// - The lifespan must be greater than or equal to the minimum lifespan.
    /// - The lifespan must be less than or equal to the maximum lifespan.
    ///
    /// **Parameters**
    /// - `proposer` - The proposer of the contract.
    /// - `goal` - The total amount of lamports the contract wishes to raise before the lifespan ends.
    /// - `lifespan` - The lifespan during which the contract must reach its goal.
    /// - `refund_bonus` - The amount of lamports to be paid to the backers if the contract fails to reach its goal before the lifespan ends.
    /// - `system_program` - The system program.
    /// - `pda_account` - The PDA account of the DAC program.
    /// - `nft_mint` - The mint of the NFT that is minted to the backer when they pledge to the contract.
    ///
    /// **Errors**
    /// - `ContractAlreadyInitialized` - Cannot initialize an already initialized contract.
    /// - `GoalLessThanMinimum` - Cannot initialize a contract with a goal less than the minimum goal.
    /// - `RefundBonusLessThanMinimum` - Cannot initialize a contract with a refund bonus less than the minimum refund bonus.
    /// - `GoalLessThanRefundBonus` - Cannot initialize a contract with a goal less than the refund bonus.
    /// - `LifespanLessThanMinimum` - Cannot initialize a contract with a lifespan less than the minimum lifespan.
    /// - `LifespanLGreaterThanMaximum` - Cannot initialize a contract with a lifespan greater than the maximum lifespan.
    /// - `NotEnoughLamportsForRefundBonus` - The proposer does not have enough lamports to cover the refund bonus.
    /// - `ArithmeticOverflow` - The operation would cause an overflow.
    pub fn new_contract<'info>(
        &mut self,
        proposer: AccountInfo<'info>,
        goal: u64,
        lifespan: u64,
        refund_bonus: u64,
        system_program: AccountInfo<'info>,
        pda_account: AccountInfo<'info>,
        nft_mint: Pubkey,
    ) -> Result<()> {
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        require_eq!(
            self.state.clone(),
            ContractState::Uninitialized,
            ContractError::ContractAlreadyInitialized
        );
        require_gte!(goal, Self::MINIMUM_GOAL, ContractError::GoalLessThanMinimum);
        require_gte!(
            refund_bonus,
            Self::MINIMUM_REFUND_BONUS,
            ContractError::RefundBonusLessThanMinimum
        );
        require_gt!(goal, refund_bonus, ContractError::GoalLessThanRefundBonus);
        require_gt!(
            lifespan,
            Self::MINIMUM_LIFESPAN,
            ContractError::LifespanLessThanMinimum
        );
        require_gte!(
            Self::MAXIMUM_LIFESPAN,
            lifespan,
            ContractError::LifespanLGreaterThanMaximum
        );
        require_gte!(
            proposer.lamports(),
            refund_bonus,
            ContractError::NotEnoughLamportsForRefundBonus
        );
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Transfer Refund Bonus >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        system_program::transfer(
            CpiContext::new(
                system_program,
                system_program::Transfer {
                    from: proposer.clone(),
                    to: pda_account,
                },
            ),
            refund_bonus,
        )?;
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Update Storage >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        self.proposer = *proposer.key;
        self.goal = goal;
        self.lifespan = lifespan;
        self.refund_bonus = refund_bonus;
        self.total_pledged = 0u64;
        self.total_backers = 0u64;
        self.state = ContractState::Active;
        self.nft_mint = nft_mint;

        // TODO: logic for initialize nft mint authority.

        Ok(())
    }

    /// Makes a pledge to the contract and transfers the pledge from the backer to the contract.
    ///
    /// **Requirements**
    /// - The contract must be active.
    /// - The contract must not have passed its deadline.
    /// - The amount must be greater than or equal to the minimum pledge.
    /// - The backer must have enough lamports to cover the pledge.
    ///
    /// **Parameters**
    /// - `backer` - The backer of the contract.
    /// - `amount` - The amount of lamports to pledge.
    /// - `system_program` - The system program.
    /// - `pda_account` - The PDA account of the DAC program.
    ///
    /// **Errors**
    /// - `ContractNotActive` - Cannot pledge to a contract that is not active.
    /// - `DeadlinePassed` - Cannot pledge to a contract that has passed its deadline.
    /// - `PledgeLessThanMinimum` - Cannot pledge less than the minimum pledge.
    /// - `NotEnoughLamportsForPledge` - The backer does not have enough lamports to cover the pledge.
    /// - `ArithmeticOverflow` - The operation would cause an overflow.
    pub fn pledge<'info>(
        &mut self,
        backer: AccountInfo<'info>,
        amount: u64,
        system_program: AccountInfo<'info>,
        pda_account: AccountInfo<'info>,
    ) -> Result<()> {
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        require_eq!(
            self.state.clone(),
            ContractState::Active,
            ContractError::ContractNotActive
        );
        require_gt!(
            self.lifespan,
            Clock::get()?
                .unix_timestamp
                .checked_sub(self.birth.try_into().unwrap())
                .ok_or(ContractError::ArithmeticOverflow)?
                .to_u64()
                .ok_or(ContractError::ArithmeticOverflow)?,
            ContractError::DeadlinePassed
        );
        require_gte!(
            amount,
            Self::MINIMUM_PLEDGE,
            ContractError::PledgeLessThanMinimum
        );
        require_gte!(
            backer.lamports(),
            amount,
            ContractError::NotEnoughLamportsForPledge
        );
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Transfer Pledge >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        system_program::transfer(
            CpiContext::new(
                system_program,
                system_program::Transfer {
                    from: backer.clone(),
                    to: pda_account,
                },
            ),
            amount,
        )?;
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Update Storage >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        self.total_pledged += amount;
        self.total_backers += 1u64;

        // TODO: logic for minting nft for backer.

        Ok(())
    }

    /// Closes the contract.
    /// Transfers the refund bonus and all pledged funds to the proposer if the contract reached its goal.
    /// Otherwise, wait for the backers to redeem their refund bonus.
    ///
    /// **Requirements**
    /// - The contract must be active.
    /// - The contract must have passed its deadline.
    ///
    /// **Parameters**
    /// - `proposer` - The proposer of the contract.
    /// - `system_program` - The system program.
    /// - `pda_account` - The PDA account of the DAC program.
    ///
    /// **Errors**
    /// - `ContractNotActive` - Cannot close a contract that is not active.
    /// - `CloseBeforeDeadline` - Cannot close a contract that has not passed its deadline.
    /// - `ArithmeticOverflow` - The operation would cause an overflow.
    pub fn close_contract<'info>(
        &mut self,
        proposer: AccountInfo<'info>,
        system_program: AccountInfo<'info>,
        pda_account: AccountInfo<'info>,
    ) -> Result<()> {
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        require_eq!(
            self.state.clone(),
            ContractState::Active,
            ContractError::ContractNotActive
        );
        require_gte!(
            Clock::get()?
                .unix_timestamp
                .checked_sub(self.birth.try_into().unwrap())
                .ok_or(ContractError::ArithmeticOverflow)?
                .to_u64()
                .ok_or(ContractError::ArithmeticOverflow)?,
            self.lifespan,
            ContractError::CloseBeforeDeadline
        ); // (Maybe contract should be allowed to close before the deadline by the proposer?)

        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Update Storage >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        if self.total_pledged >= self.goal {
            // goal reached, transfer all funds to proposer.
            system_program::transfer(
                CpiContext::new(
                    system_program,
                    system_program::Transfer {
                        from: pda_account,
                        to: proposer,
                    },
                ),
                self.refund_bonus
                    .checked_add(self.total_pledged)
                    .ok_or(ContractError::ArithmeticOverflow)?,
            )?;
            self.state = ContractState::Closed { funded: Some(()) };
        } else {
            self.state = ContractState::Closed { funded: None };
        }

        // TODO: logic for nft mint closing logic.
        Ok(())
    }

    // TODO: Documentation
    pub fn redeem_refund_bonus<'info>(&mut self) -> Result<()> {
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> Validation >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        require_eq!(
            self.state.clone(),
            ContractState::Closed { funded: None },
            ContractError::ContractNotAvailableForRedeem
        );

        // TODO: logic for burning nft and transferring refund bonus accordingly to backer.

        Ok(())
    }

    // --------------------------------- Getter Functions ------------------------------------------
    pub fn get_state(&self) -> ContractState {
        self.state.clone()
    }

    pub fn get_proposer(&self) -> Pubkey {
        self.proposer.clone()
    }

    pub fn get_goal(&self) -> u64 {
        self.goal
    }

    pub fn get_birth(&self) -> u64 {
        self.birth
    }

    pub fn get_lifespan(&self) -> u64 {
        self.lifespan
    }

    pub fn get_refund_bonus(&self) -> u64 {
        self.refund_bonus
    }

    pub fn get_total_pledged(&self) -> u64 {
        self.total_pledged
    }

    pub fn get_total_backers(&self) -> u64 {
        self.total_backers
    }

    pub fn get_nft_mint(&self) -> Pubkey {
        self.nft_mint.clone()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ContractState {
    Uninitialized,
    Active,
    Closed { funded: Option<()> },
}
impl Default for ContractState {
    fn default() -> Self {
        ContractState::Uninitialized
    }
}
impl ToString for ContractState {
    fn to_string(&self) -> String {
        match self {
            ContractState::Uninitialized => "Uninitialized".to_string(),
            ContractState::Active => "Active".to_string(),
            ContractState::Closed { funded: s } => {
                if s.is_some() {
                    "Closed (Funded)".to_string()
                } else {
                    "Closed (Failed)".to_string()
                }
            }
        }
    }
}
