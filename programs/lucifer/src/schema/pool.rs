use crate::constant::*;
use crate::f64_trait::F64Trait;
use anchor_lang::prelude::*;
use num_traits::ToPrimitive;

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub base_mint: Pubkey,
    // PDAs
    pub stable_mint: Pubkey,
    pub lpt_mint: Pubkey,
    pub treasurer: Pubkey,
    //
    pub balance: u64,
    pub stable_balance: u64,
    pub base_balance: u64,
    pub fee: u64,
    pub total_lpt_fee: u64,
    pub lpt_supply: u64,
    pub start_time: i64,
}

impl Pool {
    pub const LEN: usize = ACCOUNT_DISCRIMINATOR
        + PUBLIC_KEY_SIZE * 6
        + U64_SIZE // balance
        + U64_SIZE // stable_balance
        + U64_SIZE // base_balance
        + U64_SIZE // fee
        + U64_SIZE // total_lpt_fee
        + U64_SIZE // lpt_supply
        + U64_SIZE; // start_time

    pub fn calc_fee(&self, amount: u64) -> Option<u64> {
        let amount_f64 = amount.to_f64()?;
        let fee_rate = self.fee.to_f64()?.checked_div(PRECISION)?;
        return Some((amount_f64.checked_mul(fee_rate)?).to_u64()?);
    }
}
