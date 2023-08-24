use crate::state::DominantAssuranceContract;
use anchor_lang::prelude::*;

pub fn pledge(ctx: Context<Pledge>, amount: u64) -> Result<()> {
    ctx.accounts.contract.clone().pledge(
        ctx.accounts.backer.to_account_info(),
        amount,
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.contract.clone().to_account_info(),
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct Pledge<'info> {
    #[account(mut)]
    pub contract: Account<'info, DominantAssuranceContract>,
    #[account(mut)]
    pub backer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
