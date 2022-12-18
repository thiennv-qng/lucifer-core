use crate::instructions::*;
use crate::oracle::*;
use crate::schema::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct Buy<'info> {
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

pub fn exec(ctx: Context<Buy>, stable_amount: u64, base_amount: u64) -> Result<()> {
  let mut total_stable_amount = stable_amount;
  // Call inner instructions Mint Stable
  if base_amount > 0 {
    let accounts = &mut MintStable {
      authority: ctx.accounts.authority.clone(),
      pool: ctx.accounts.pool.clone(),
      treasurer: ctx.accounts.treasurer.clone(),
      base_mint: ctx.accounts.base_mint.clone(),
      stable_mint: ctx.accounts.stable_mint.clone(),
      base_treasury: ctx.accounts.base_treasury.clone(),
      stable_treasury: ctx.accounts.stable_treasury.clone(),
      base_token_account: ctx.accounts.base_token_account.clone(),
      stable_token_account: ctx.accounts.stable_token_account.clone(),
      system_program: ctx.accounts.system_program.clone(),
      token_program: ctx.accounts.token_program.clone(),
      associated_token_program: ctx.accounts.associated_token_program.clone(),
      rent: ctx.accounts.rent.clone(),
    };
    let mint_to_context = Context::new(&ctx.program_id, accounts, &[], ctx.bumps.clone());
    mint_stable::exec(mint_to_context, base_amount).unwrap();
    ctx.accounts.pool.reload()?;
    total_stable_amount += base_amount;
  }
  //
  let pool = &mut ctx.accounts.pool;
  // Transfer Mint
  let amount = calc_ask_amount_swap(
    total_stable_amount,
    pool.stable_balance,
    pool.balance,
    pool.fee,
  )
  .unwrap();
  let amount_ignore_fee =
    calc_ask_amount_swap(total_stable_amount, pool.stable_balance, pool.balance, 0).unwrap();
  // Transfer Stable Mint
  msg!("11");
  if total_stable_amount > 0 {
    token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.stable_token_account.to_account_info(),
          to: ctx.accounts.stable_treasury.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
        },
      ),
      total_stable_amount,
    )?;
  }
  pool.stable_balance += total_stable_amount;
  pool.balance -= amount;
  // Transfer Mint
  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];
  msg!("22");
  token::transfer(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.treasury.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.treasurer.to_account_info(),
      },
      seeds,
    ),
    amount,
  )?;
  // Update Fee
  let amounts = vec![amount_ignore_fee - amount, 0];
  let reserves = vec![pool.balance, pool.stable_balance];
  let supply = ctx.accounts.lpt_mint.supply;
  pool.total_lpt_fee +=
    calc_lpt_receive_add_full_side(supply, &amounts, &reserves, pool.fee).unwrap();
  Ok(())
}
