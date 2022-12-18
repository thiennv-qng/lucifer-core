use anchor_lang::prelude::*;

declare_id!("B1hw4bqgqEsu7okvd6efKg2oKudtw4yNCRbV2qYWkUp9");

pub mod math;
pub use math::*;

pub mod schema;
pub use schema::*;

pub mod instructions;
pub use instructions::*;

pub mod errors;
pub use errors::*;

pub mod utils;
pub use utils::*;

#[program]
pub mod lucifer {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee: u64,
        amount: u64,
        stable_amount: u64,
        base_amount: u64,
    ) -> Result<()> {
        initialize_pool::exec(ctx, fee, amount, stable_amount, base_amount)
    }

    pub fn mint_stable(ctx: Context<MintStable>, base_amount: u64) -> Result<()> {
        mint_stable::exec(ctx, base_amount)
    }
    pub fn burn_stable(ctx: Context<BurnStable>, stable_amount: u64) -> Result<()> {
        burn_stable::exec(ctx, stable_amount)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount: u64,
        stable_amount: u64,
        base_amount: u64,
    ) -> Result<()> {
        add_liquidity::exec(ctx, amount, stable_amount, base_amount)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lpt_amount: u64) -> Result<()> {
        remove_liquidity::exec(ctx, lpt_amount)
    }

    pub fn borrow(ctx: Context<Borrow>, lpt_amount: u64) -> Result<()> {
        borrow::exec(ctx, lpt_amount)
    }

    pub fn repay(ctx: Context<Repay>) -> Result<()> {
        repay::exec(ctx)
    }
    pub fn buy(ctx: Context<Buy>, stable_amount: u64, base_amount: u64) -> Result<()> {
        buy::exec(ctx, stable_amount, base_amount)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64) -> Result<()> {
        sell::exec(ctx, amount)
    }

    pub fn initialize_jupiter(ctx: Context<InitializeJupiter>) -> Result<()> {
        initialize_jupiter::exec(ctx)
    }

    pub fn swap_jupiter(ctx: Context<SwapJupiter>, amount_in: u64, amount_out: u64) -> Result<()> {
        swap_jupiter::exec(ctx, amount_in, amount_out)
    }
}
