use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::CustomError;

#[account]
pub struct AirdropPool {
    pub authority: Pubkey,
}

/// Versioned accounting is intentionally separate from `AirdropPool` so an
/// upgrade does not change the layout of already-initialized pool accounts.
#[account]
pub struct PoolAccounting {
    pub pool_authority: Pubkey,
    pub total_allocated: u64,
    pub total_claimed: u64,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<AirdropPool>(),
        seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = authority,
    )]
    pub from: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool_authority,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        constraint = mint.mint_authority == COption::Some(authority.key()) @ CustomError::InvalidMintAuthority,
    )]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
