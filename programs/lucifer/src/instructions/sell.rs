use crate::oracle::*;
use crate::schema::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct Sell<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  // Pool's info
  #[account(mut, has_one=mint)]
  pub pool: Account<'info, Pool>,
  #[account(seeds = [b"treasurer", &pool.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  // Pool's Mints
  pub mint: Box<Account<'info, token::Mint>>,
  #[account(
    mut,
    seeds = [b"stable_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub stable_mint: Account<'info, token::Mint>,
  pub base_mint: Box<Account<'info, token::Mint>>,
  #[account(
    mut,
    seeds = [b"lpt_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub lpt_mint: Account<'info, token::Mint>,
  // Pool's token account
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = treasurer
  )]
  pub treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = stable_mint,
    associated_token::authority = treasurer
  )]
  pub stable_treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = base_mint,
    associated_token::authority = treasurer
  )]
  pub base_treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = lpt_mint,
    associated_token::authority = treasurer
  )]
  pub lpt_treasury: Box<Account<'info, token::TokenAccount>>,
  // Wallet's Token Accounts
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = mint,
    associated_token::authority = authority
  )]
  pub token_account: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = stable_mint,
    associated_token::authority = authority
  )]
  pub stable_token_account: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = base_mint,
    associated_token::authority = authority
  )]
  pub base_token_account: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = lpt_mint,
    associated_token::authority = authority
  )]
  pub lpt_token_account: Box<Account<'info, token::TokenAccount>>,

  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<Sell>, amount: u64) -> Result<()> {
  //
  let pool = &mut ctx.accounts.pool;
  // Transfer Mint
  let stable_amount =
    calc_ask_amount_swap(amount, pool.balance, pool.stable_balance, pool.fee).unwrap();
  let stable_amount_ignore_fee =
    calc_ask_amount_swap(amount, pool.balance, pool.stable_balance, 0).unwrap();

  // Transfer Mint
  token::transfer(
    CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.token_account.to_account_info(),
        to: ctx.accounts.treasury.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      },
    ),
    amount,
  )?;

  pool.balance += amount;
  pool.stable_balance -= stable_amount;

  // Transfer Mint
  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];
  token::transfer(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.stable_treasury.to_account_info(),
        to: ctx.accounts.stable_token_account.to_account_info(),
        authority: ctx.accounts.treasurer.to_account_info(),
      },
      seeds,
    ),
    stable_amount,
  )?;
  // Update Fee
  let amounts = vec![0, stable_amount_ignore_fee - stable_amount];
  let reserves = vec![pool.balance, pool.stable_balance];
  let supply = ctx.accounts.lpt_mint.supply;
  pool.total_lpt_fee +=
    calc_lpt_receive_add_full_side(supply, &amounts, &reserves, pool.fee).unwrap();
  Ok(())
}
