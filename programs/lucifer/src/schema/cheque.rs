use crate::constant::*;
use anchor_lang::prelude::*;

#[account]
pub struct Cheque {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub borrow_amount: u64,
    pub base_amount: u64,
}

impl Cheque {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR + PUBLIC_KEY_SIZE * 2 + U64_SIZE * 3;
}
