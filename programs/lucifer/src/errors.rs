use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Over borrow limit!")]
    OverBorrow,
    #[msg("Required greater than zero")]
    AmountZero,
    #[msg("Slippage error")]
    Slippage,
    #[msg("Invalid Amount")]
    InvalidAmount,
}
