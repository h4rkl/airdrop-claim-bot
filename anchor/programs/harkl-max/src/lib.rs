use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use errors::*;
use state::*;

pub mod errors;
pub mod state;

declare_id!("HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ");

#[program]
pub mod airdrop {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

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

        ctx.accounts.pool_authority.set_inner(AirdropPool {
            authority: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Creates one immutable, fixed-size allocation for a recipient.
    /// Only the pool authority can call this instruction.
    pub fn register_claim(ctx: Context<RegisterClaim>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

        let pool_accounting = &mut ctx.accounts.pool_accounting;
        if pool_accounting.pool_authority == Pubkey::default() {
            pool_accounting.pool_authority = ctx.accounts.pool_authority.key();
        }
        require_keys_eq!(
            pool_accounting.pool_authority,
            ctx.accounts.pool_authority.key(),
            CustomError::InvalidClaimAllocation
        );

        let total_allocated = pool_accounting
            .total_allocated
            .checked_add(amount)
            .ok_or(CustomError::MathOverflow)?;

        // The PDA-owned token account has no withdrawal instruction. Reserving
        // allocations against its current balance prevents an insolvent airdrop.
        require!(
            total_allocated <= ctx.accounts.pool_token_account.amount,
            CustomError::InsufficientPoolBalance
        );

        ctx.accounts.claim_allocation.set_inner(ClaimAllocation {
            user: ctx.accounts.user.key(),
            pool_authority: ctx.accounts.pool_authority.key(),
            amount,
            has_claimed: false,
        });
        pool_accounting.total_allocated = total_allocated;

        Ok(())
    }

    /// Transfers exactly the allocation that the pool authority registered for
    /// the signing recipient. The recipient never supplies a claim amount.
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        let claim_allocation = &mut ctx.accounts.claim_allocation;
        let mint = ctx.accounts.mint.key();
        let amount = claim_allocation.amount;

        require!(!claim_allocation.has_claimed, CustomError::AlreadyClaimed);
        require!(amount > 0, CustomError::InvalidAmount);

        let pool_accounting = &mut ctx.accounts.pool_accounting;
        pool_accounting.total_claimed = pool_accounting
            .total_claimed
            .checked_add(amount)
            .ok_or(CustomError::MathOverflow)?;
        require!(
            pool_accounting.total_claimed <= pool_accounting.total_allocated,
            CustomError::InvalidClaimAllocation
        );

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

        claim_allocation.has_claimed = true;

        Ok(())
    }

    /// Cancels an unclaimed allocation, releasing its reserved balance back to
    /// the pool administrator. This also lets an administrator correct a
    /// mistyped recipient address.
    pub fn revoke_claim(ctx: Context<RevokeClaim>) -> Result<()> {
        let claim_allocation = &ctx.accounts.claim_allocation;
        require!(!claim_allocation.has_claimed, CustomError::AlreadyClaimed);

        let pool_accounting = &mut ctx.accounts.pool_accounting;
        pool_accounting.total_allocated = pool_accounting
            .total_allocated
            .checked_sub(claim_allocation.amount)
            .ok_or(CustomError::MathOverflow)?;

        Ok(())
    }

    /// Returns tokens that are not reserved by an outstanding allocation to
    /// the pool authority. Reserved allocations can never be withdrawn.
    pub fn withdraw_unallocated(ctx: Context<WithdrawUnallocated>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

        let pool_accounting = &mut ctx.accounts.pool_accounting;
        if pool_accounting.pool_authority == Pubkey::default() {
            pool_accounting.pool_authority = ctx.accounts.pool_authority.key();
        }
        require_keys_eq!(
            pool_accounting.pool_authority,
            ctx.accounts.pool_authority.key(),
            CustomError::InvalidClaimAllocation
        );

        let outstanding = pool_accounting
            .total_allocated
            .checked_sub(pool_accounting.total_claimed)
            .ok_or(CustomError::MathOverflow)?;
        let withdrawable = ctx
            .accounts
            .pool_token_account
            .amount
            .checked_sub(outstanding)
            .ok_or(CustomError::InsufficientPoolBalance)?;
        require!(amount <= withdrawable, CustomError::InsufficientPoolBalance);

        let mint = ctx.accounts.mint.key();
        let seeds = &[
            mint.as_ref(),
            AIRDROP_PROTOCOL.as_ref(),
            &[ctx.bumps.pool_authority],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        Ok(())
    }
}
