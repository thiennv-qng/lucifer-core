use crate::oracle::*;
use crate::schema::pool::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct BurnStable<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  // Pool's info
  #[account(mut, has_one=base_mint)]
  pub pool: Account<'info, Pool>,
  #[account(seeds = [b"treasurer", &pool.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  // Pool's Mints
  pub base_mint: Box<Account<'info, token::Mint>>,
  #[account(
    mut,
    seeds = [b"stable_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub stable_mint: Account<'info, token::Mint>,
  #[account(
    mut,
    seeds = [b"lpt_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub lpt_mint: Account<'info, token::Mint>,
  // Pool's token account
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = base_mint,
    associated_token::authority = treasurer
  )]
  pub base_treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = stable_mint,
    associated_token::authority = treasurer
  )]
  pub stable_treasury: Box<Account<'info, token::TokenAccount>>,
  // Wallet's Token Accounts
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = base_mint,
    associated_token::authority = authority
  )]
  pub base_token_account: Box<Account<'info, token::TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = stable_mint,
    associated_token::authority = authority
  )]
  pub stable_token_account: Box<Account<'info, token::TokenAccount>>,

  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<BurnStable>, stable_amount: u64) -> Result<()> {
  let pool = &mut ctx.accounts.pool;
  // Burn stable mint
  let burn_stable = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    token::Burn {
      from: ctx.accounts.stable_token_account.to_account_info(),
      mint: ctx.accounts.stable_mint.to_account_info(),
      authority: ctx.accounts.authority.to_account_info(),
    },
  );
  token::burn(burn_stable, stable_amount)?;
  msg!("burn stable {}", stable_amount);
  // Transfer base mint
  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];
  let stable_amount_after_fee = pool.calc_fee(stable_amount).unwrap();
  token::transfer(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.base_treasury.to_account_info(),
        to: ctx.accounts.base_token_account.to_account_info(),
        authority: ctx.accounts.treasurer.to_account_info(),
      },
      seeds,
    ),
    stable_amount_after_fee,
  )?;
  pool.base_balance -= stable_amount_after_fee;
  // Update Fee
  let amounts = vec![0, stable_amount - stable_amount_after_fee];
  let reserves = vec![pool.balance, pool.stable_balance];
  let supply = ctx.accounts.lpt_mint.supply;
  pool.total_lpt_fee +=
    calc_lpt_receive_add_full_side(supply, &amounts, &reserves, pool.fee).unwrap();

  Ok(())
}
