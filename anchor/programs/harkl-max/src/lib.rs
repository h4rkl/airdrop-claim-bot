use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
mod airdrop {
    use super::*;

    pub const AIRDROP_PROTOCOL: &[u8] = b"airdrop_protocol";

    // Initialize the airdrop pool account with a certain amount of tokens
    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        ctx.accounts.pool.authority = *ctx.accounts.authority.key;

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
        require!(
            !ctx.accounts.user_claim.has_claimed,
            CustomError::AlreadyClaimed
        );
        require!(amount > 1_000_000_000, CustomError::InvalidAmount);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
            ),
            amount,
        )?;

        ctx.accounts.user_claim.has_claimed = true;

        Ok(())
    }
}

// Define the account structures
#[account]
pub struct AirdropPool {
    pub authority: Pubkey,
}

#[account]
pub struct UserClaim {
    pub user: Pubkey,
    pub has_claimed: bool,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + 32, seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL], bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [pool.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL], bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [pool.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserClaim>(),
        seeds = [user.key().as_ref(), pool.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub user_claim: Account<'info, UserClaim>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(seeds = [AIRDROP_PROTOCOL], bump)]
    pub pool_authority: UncheckedAccount<'info>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Custom error for handling already claimed tokens
#[error_code]
pub enum CustomError {
    #[msg("Invalid pool token account.")]
    InvalidTokenPoolAccount,
    #[msg("Invalid pool address.")]
    InvalidPoolAddress,
    #[msg("User has already claimed their tokens.")]
    AlreadyClaimed,
    #[msg("Invalid amount.")]
    InvalidAmount,
}
