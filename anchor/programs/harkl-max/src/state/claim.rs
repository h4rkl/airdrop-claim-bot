use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::CustomError;
use crate::state::{AirdropPool, PoolAccounting};

#[account]
pub struct ClaimAllocation {
    pub user: Pubkey,
    pub pool_authority: Pubkey,
    pub amount: u64,
    pub has_claimed: bool,
}

#[derive(Accounts)]
pub struct RegisterClaim<'info> {
    #[account(
        mut,
        seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        has_one = authority @ CustomError::Unauthorized,
    )]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool_authority,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + std::mem::size_of::<PoolAccounting>(),
        seeds = [pool_authority.key().as_ref(), b"pool_accounting"],
        bump,
    )]
    pub pool_accounting: Account<'info, PoolAccounting>,
    /// CHECK: The allocation PDA binds this key, and it must sign to claim.
    pub user: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<ClaimAllocation>(),
        seeds = [user.key().as_ref(), pool_authority.key().as_ref(), b"claim_allocation"],
        bump,
    )]
    pub claim_allocation: Account<'info, ClaimAllocation>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(
        mut,
        seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool_authority,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), b"pool_accounting"],
        bump,
        has_one = pool_authority @ CustomError::InvalidClaimAllocation,
    )]
    pub pool_accounting: Account<'info, PoolAccounting>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), pool_authority.key().as_ref(), b"claim_allocation"],
        bump,
        has_one = user @ CustomError::InvalidClaimAllocation,
        has_one = pool_authority @ CustomError::InvalidClaimAllocation,
    )]
    pub claim_allocation: Account<'info, ClaimAllocation>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RevokeClaim<'info> {
    #[account(
        seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        has_one = authority @ CustomError::Unauthorized,
    )]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), b"pool_accounting"],
        bump,
        has_one = pool_authority @ CustomError::InvalidClaimAllocation,
    )]
    pub pool_accounting: Account<'info, PoolAccounting>,
    /// CHECK: The allocation PDA binds this key and the authority may revoke it.
    pub user: UncheckedAccount<'info>,
    #[account(
        mut,
        close = authority,
        seeds = [user.key().as_ref(), pool_authority.key().as_ref(), b"claim_allocation"],
        bump,
        has_one = user @ CustomError::InvalidClaimAllocation,
        has_one = pool_authority @ CustomError::InvalidClaimAllocation,
    )]
    pub claim_allocation: Account<'info, ClaimAllocation>,
    pub mint: Box<Account<'info, Mint>>,
}

#[derive(Accounts)]
pub struct WithdrawUnallocated<'info> {
    #[account(
        seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        has_one = authority @ CustomError::Unauthorized,
    )]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool_authority,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + std::mem::size_of::<PoolAccounting>(),
        seeds = [pool_authority.key().as_ref(), b"pool_accounting"],
        bump,
    )]
    pub pool_accounting: Account<'info, PoolAccounting>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = authority,
    )]
    pub destination: Account<'info, TokenAccount>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
