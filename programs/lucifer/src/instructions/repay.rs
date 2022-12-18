use crate::errors::ErrorCode;
use crate::f64_trait::F64Trait;
use crate::schema::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};
use num_traits::ToPrimitive;

#[derive(Accounts)]
pub struct Repay<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  // Pool's info
  #[account(mut)]
  pub pool: Account<'info, Pool>,
  #[account(seeds = [b"treasurer", &pool.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  // Pool's Mints
  #[account(mut)]
  pub base_mint: Account<'info, token::Mint>,
  #[account(
    mut,
    seeds = [b"lpt_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub lpt_mint: Account<'info, token::Mint>,
  // Pool's token account
  #[account(
    mut,
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
  // Instruction Data
  #[account(
    init_if_needed,
    payer = authority,
    space = Cheque::LEN,
    seeds = [b"cheque".as_ref(), &pool.key().to_bytes(), &authority.key().to_bytes()], bump
  )]
  pub cheque: Account<'info, Cheque>,
  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<Repay>) -> Result<()> {
  let pool = &mut ctx.accounts.pool;
  let cheque = &mut ctx.accounts.cheque;

  // Wallet Actions: Transfer repay token
  token::transfer(
    CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.base_token_account.to_account_info(),
        to: ctx.accounts.base_treasury.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      },
    ),
    cheque.base_amount,
  )?;
  cheque.base_amount = 0;
  // Pool Actions: Transfer lpt token

  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];
  token::transfer(
    CpiContext::new_with_signer(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.lpt_treasury.to_account_info(),
        to: ctx.accounts.lpt_token_account.to_account_info(),
        authority: ctx.accounts.treasurer.to_account_info(),
      },
      seeds,
    ),
    cheque.borrow_amount,
  )?;
  cheque.borrow_amount = 0;
  Ok(())
}
