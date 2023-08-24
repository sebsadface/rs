use crate::state::DominantAssuranceContract;
use anchor_lang::prelude::*;

pub fn close_contract(ctx: Context<CloseContract>) -> Result<()> {
    ctx.accounts.contract.close_contract(
        ctx.accounts.proposer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.pda_account.to_account_info(),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct CloseContract<'info> {
    #[account(mut)]
    pub contract: Account<'info, DominantAssuranceContract>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK:
    pub pda_account: AccountInfo<'info>,
}
