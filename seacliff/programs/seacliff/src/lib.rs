use anchor_lang::prelude::*;
use instructions::*;

pub mod errors;
pub mod instructions;
pub mod state;

declare_id!("4mBgVuZGzbLQ5SQVU8KVoqmv19Aq9HzDXgovdKyzzrsy");

#[program]
pub mod seacliff {
    use super::*;

    pub fn create_contract(
        ctx: Context<CreateContract>,
        goal: u64,
        lifespan: u64,
        refund_bonus: u64,
    ) -> Result<()> {
        msg!("You called create_contract");
        instructions::create_contract::create_contract(ctx, goal, lifespan, refund_bonus)
    }

    pub fn pledge(ctx: Context<Pledge>, amount: u64) -> Result<()> {
        msg!("You called pledge");
        instructions::pledge::pledge(ctx, amount)
    }

    pub fn close_contract(ctx: Context<CloseContract>) -> Result<()> {
        msg!("You called close_contract");
        instructions::close_contract::close_contract(ctx)
    }

    pub fn redeem_refund_bonus(ctx: Context<RedeemRefundBonus>) -> Result<()> {
        msg!("You called redeem_refund_bonus");
        instructions::redeem_refund_bonus::redeem_refund_bonus(ctx)
    }
}
