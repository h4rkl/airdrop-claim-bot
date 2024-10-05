use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
mod airdrop {
    use token::spl_token;

    use super::*;

    pub const MINT_AIRDROP_POOL: &[u8] = b"mint_airdrop_pool";
    pub const POOL_TOKEN_ACCOUNT: &[u8] = b"pool_token_account";

    // Initialize the airdrop pool account with a certain amount of tokens
    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        let (pool_pda, _bump) = Pubkey::find_program_address(
            &[ctx.accounts.mint.key().as_ref(), MINT_AIRDROP_POOL],
            &crate::ID,
        );

        require!(
            ctx.accounts.pool.key() == pool_pda,
            CustomError::InvalidPoolAddress
        );

        let pool_token_account = anchor_spl::associated_token::get_associated_token_address(
            &pool_pda,
            &ctx.accounts.mint.key(),
        );

        require!(   
            ctx.accounts.pool_token_account.key() == pool_token_account,
            CustomError::InvalidPoolTokenAccount
        );

        // Create the associated token account for the pool if it doesn't exist
        if ctx.accounts.pool_token_account.amount == 0 {
            anchor_spl::associated_token::create(CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: ctx.accounts.authority.to_account_info(),
                    associated_token: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ))?;
        }

        ctx.accounts.pool.authority = *ctx.accounts.authority.key;

        // Transfer tokens to the pool's token account
        token::transfer(ctx.accounts.into_transfer_to_pool_context(), amount)?;

        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, amount: u64) -> Result<()> {
        let mint_key = ctx.accounts.mint.key();
        let pool_key = ctx.accounts.pool.key();

        let pool_bump = ctx.bumps.pool;
        let pool_token_account_bump = ctx.bumps.pool_token_account;

        // Use the long-lived variables in the seeds array
        let pool_seeds = &[
            mint_key.as_ref(),
            MINT_AIRDROP_POOL.as_ref(),
            &[pool_bump],
        ];

        let pool_token_account_seeds = &[
            pool_key.as_ref(),
            mint_key.as_ref(),
            POOL_TOKEN_ACCOUNT,
            &[pool_token_account_bump],
        ];

        // Construct the transfer instruction
        let transfer_instruction = spl_token::instruction::transfer(
            &spl_token::ID,
            &ctx.accounts.pool_token_account.key(),
            &ctx.accounts.user_token_account.key(),
            &ctx.accounts.pool.key(),
            &[],
            amount,
        )?;

        // Call the instruction with `invoke_signed` using correct seeds
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.pool_token_account.to_account_info(),
                ctx.accounts.user_token_account.to_account_info(),
                ctx.accounts.pool.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[pool_seeds, pool_token_account_seeds],
        )?;

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
    pub has_claimed: bool,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + 32, seeds = [mint.key().as_ref(), b"mint_airdrop_pool"], bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        seeds = [pool.key().as_ref(), mint.key().as_ref(), b"pool_token_account"],
        bump,
        token::mint = mint,
        token::authority = pool,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(seeds = [mint.key().as_ref(), b"mint_airdrop_pool"], bump)]
    pub pool: Account<'info, AirdropPool>,
    #[account(
        mut,
        seeds = [pool.key().as_ref(), mint.key().as_ref(), b"pool_token_account"],
        bump,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Custom error for handling already claimed tokens
#[error_code]
pub enum CustomError {
    #[msg("Invalid pool token account.")]
    InvalidPoolTokenAccount,
    #[msg("Invalid pool address.")]
    InvalidPoolAddress,
    #[msg("User has already claimed their tokens.")]
    AlreadyClaimed,
}
