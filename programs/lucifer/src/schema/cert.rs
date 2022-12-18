use crate::constant::*;
use anchor_lang::prelude::*;

#[account]
pub struct Cert {
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
}

impl Cert {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR + PUBLIC_KEY_SIZE * 2 + U64_SIZE;
}
