use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
pub mod harkl_max {
    use super::*;

    pub fn load_token_pool(ctx: Context<LoadTokenPool>, amount: u64) -> Result<()> {
        // Transfer tokens from the admin to the token pool account
        let transfer_instruction = Transfer {
            from: ctx.accounts.admin_token_account.to_account_info(),
            to: ctx.accounts.token_pool.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn airdrop(ctx: Context<Airdrop>, amount: u64) -> Result<()> {
        // Transfer tokens from the pool to the user's token account
        let transfer_instruction = Transfer {
            from: ctx.accounts.token_pool.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.token_pool_authority.to_account_info(),
        };

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
                &[&[b"token_pool_authority", &[ctx.bumps.token_pool_authority]]],
            ),
            amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct LoadTokenPool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = admin_token_account.mint == token_pool.mint,
        constraint = admin_token_account.owner == admin.key()
    )]
    pub admin_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = token_pool.mint == AIRDROP_TOKEN_MINT_PUBKEY
    )]
    pub token_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Airdrop<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = token_pool.mint == token_mint.key()
    )]
    pub token_pool: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"token_pool_authority"],
        bump
    )]
    pub token_pool_authority: AccountInfo<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Define the constant for your custom SPL token mint address
pub const AIRDROP_TOKEN_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([/* Your token mint address bytes */]);