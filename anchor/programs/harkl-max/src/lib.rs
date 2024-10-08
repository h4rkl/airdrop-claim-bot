use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use state::*;
use errors::*;

pub mod errors;
pub mod state;

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
pub mod airdrop {
    use super::*;

    // Initialize the airdrop pool account with a certain amount of tokens
    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        ctx.accounts.pool_authority.authority = *ctx.accounts.authority.key;

        // Transfer tokens to the pool's token account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, amount: u64) -> Result<()> {
        let user_claim = &mut ctx.accounts.user_claim;
        let mint = ctx.accounts.mint.key();

        require!(!user_claim.has_claimed, CustomError::AlreadyClaimed);
        require!(amount > 1_000_000_000, CustomError::InvalidAmount);

        // Transfer tokens
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let seeds = &[
            mint.as_ref(),
            AIRDROP_PROTOCOL.as_ref(),
            &[ctx.bumps.pool_authority],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            amount,
        )?;

        user_claim.has_claimed = true;

        Ok(())
    }
}
