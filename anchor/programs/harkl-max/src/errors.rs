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
}