use crate::constant::*;
use anchor_lang::prelude::*;

// Test for Devnet
#[account]
pub struct Jupiter {
    pub base_mint: Pubkey,
}

impl Jupiter {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR + PUBLIC_KEY_SIZE * 2;
}
