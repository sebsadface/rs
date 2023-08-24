use crate::state::DominantAssuranceContract;
use anchor_lang::prelude::*;
use anchor_spl::token::{InitializeMint, Mint, Token};

pub fn create_contract(
    ctx: Context<CreateContract>,
    goal: u64,
    lifespan: u64,
    refund_bonus: u64,
) -> Result<()> {
    let (mint_authority, _bump_seed) =
        Pubkey::find_program_address(&[&ctx.accounts.proposer.key.to_bytes()], ctx.program_id);

    // Define the mint initialization context
    let mint_cpi_accounts = InitializeMint {
        mint: ctx.accounts.nft_mint.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let mint_cpi_program = ctx.accounts.token_program.clone();
    let mint_cpi_ctx = CpiContext::new(mint_cpi_program.to_account_info(), mint_cpi_accounts);
    let decimals = 0;

    // Initialize the mint with the derived PDA as the mint authority
    anchor_spl::token::initialize_mint(mint_cpi_ctx, decimals, &mint_authority, None)?;

    let contract_info = ctx.accounts.contract.to_account_info();
    ctx.accounts.contract.new_contract(
        ctx.accounts.proposer.to_account_info(),
        goal,
        lifespan,
        refund_bonus,
        ctx.accounts.system_program.to_account_info(),
        contract_info,
        ctx.accounts.nft_mint.key(),
    )
}

#[derive(Accounts)]
pub struct CreateContract<'info> {
    #[account(init, payer = proposer, space = DominantAssuranceContract::MAXIMUM_SIZE + 8)]
    pub contract: Account<'info, DominantAssuranceContract>,
    #[account(init, payer = proposer, space = Mint::LEN)]
    pub nft_mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
