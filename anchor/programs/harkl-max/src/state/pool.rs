use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;

#[account]
pub struct AirdropPool {
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + 32, seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL], bump)]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
        token::mint = mint,
        token::authority = pool_authority,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}