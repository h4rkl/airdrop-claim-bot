use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint, Transfer};

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
mod airdrop {
    use super::*;

    // Initialize the airdrop pool account with a certain amount of tokens
    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = *ctx.accounts.authority.key;
        pool.bump = ctx.bumps.pool;

        // Transfer the specified amount of tokens to the airdrop pool
        token::transfer(
            ctx.accounts.into_transfer_to_pool_context(),
            amount,
        )?;

        Ok(())
    }

    // Allow users to claim a specified amount of tokens
    pub fn claim_tokens(ctx: Context<ClaimTokens>, amount: u64) -> Result<()> {
        let user_claim = &mut ctx.accounts.user_claim;

        // Check if the user has already claimed tokens
        require!(!user_claim.has_claimed, CustomError::AlreadyClaimed);

        // Mark the user as having claimed their tokens
        user_claim.has_claimed = true;

        // Transfer tokens from the pool to the user
        token::transfer(
            ctx.accounts.into_transfer_to_user_context(),
            amount,
        )?;

        Ok(())
    }
}

// Define the account structures
#[account]
pub struct AirdropPool {
    pub authority: Pubkey,   // The authority (admin) of the pool
    pub bump: u8,            // PDA bump
}

#[account]
pub struct UserClaim {
    pub has_claimed: bool,
}

// Context for initializing the airdrop pool
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 1, seeds = [b"airdrop_pool"], bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializePool<'info> {
    fn into_transfer_to_pool_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info().clone(),
            to: self.pool_token_account.to_account_info().clone(),
            authority: self.authority.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

// Context for claiming tokens
#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut, seeds = [b"airdrop_pool"], bump = pool.bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(init_if_needed, payer = user, space = 8 + 1, seeds = [user.key().as_ref(), b"user_claim"], bump)]
    pub user_claim: Account<'info, UserClaim>,
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> ClaimTokens<'info> {
    fn into_transfer_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.pool_token_account.to_account_info().clone(),
            to: self.user_token_account.to_account_info().clone(),
            authority: self.pool.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

// Custom error for handling already claimed tokens
#[error_code]
pub enum CustomError {
    #[msg("User has already claimed their tokens.")]
    AlreadyClaimed,
}