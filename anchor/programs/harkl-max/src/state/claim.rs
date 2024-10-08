use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::state::AirdropPool;

#[account]
pub struct UserClaim {
    pub user: Pubkey,
    pub has_claimed: bool,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(seeds = [mint.key().as_ref(), AIRDROP_PROTOCOL], bump)]
    pub pool_authority: Account<'info, AirdropPool>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [pool_authority.key().as_ref(), mint.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserClaim>(),
        seeds = [user.key().as_ref(), pool_authority.key().as_ref(), AIRDROP_PROTOCOL],
        bump,
    )]
    pub user_claim: Account<'info, UserClaim>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}