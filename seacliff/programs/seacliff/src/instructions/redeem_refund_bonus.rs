use crate::state::DominantAssuranceContract;
use anchor_lang::prelude::*;

pub fn redeem_refund_bonus(ctx: Context<RedeemRefundBonus>) -> Result<()> {
    ctx.accounts.contract.redeem_refund_bonus()?;

    Ok(())
}

#[derive(Accounts)]
pub struct RedeemRefundBonus<'info> {
    #[account(mut)]
    pub contract: Account<'info, DominantAssuranceContract>,
    #[account(mut)]
    pub backer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
