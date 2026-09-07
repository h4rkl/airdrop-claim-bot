use anchor_lang::prelude::*;

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
    #[msg("The signer is not authorized to administer this pool.")]
    Unauthorized,
    #[msg("The pool initializer must be the mint authority.")]
    InvalidMintAuthority,
    #[msg("The pool does not have enough unallocated tokens.")]
    InsufficientPoolBalance,
    #[msg("Claim allocation does not match the claimant or pool.")]
    InvalidClaimAllocation,
    #[msg("Arithmetic overflow.")]
    MathOverflow,
}
