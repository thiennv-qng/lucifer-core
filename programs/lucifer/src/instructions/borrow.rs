use crate::errors::ErrorCode;
use crate::oracle::*;
use crate::schema::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct Borrow<'info> {
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

pub fn exec(ctx: Context<Borrow>, lpt_amount: u64) -> Result<()> {
  if !(lpt_amount > 0) {
    return err!(ErrorCode::AmountZero);
  }
  let pool = &mut ctx.accounts.pool;
  let cheque = &mut ctx.accounts.cheque;
  let lpt_amount_fee = pool.calc_fee(lpt_amount).unwrap();
  // Fee
  let lpt_amount_with_fee = lpt_amount - lpt_amount_fee;
  let burn_lpt = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    token::Burn {
      from: ctx.accounts.lpt_token_account.to_account_info(),
      mint: ctx.accounts.lpt_mint.to_account_info(),
      authority: ctx.accounts.authority.to_account_info(),
    },
  );
  token::burn(burn_lpt, lpt_amount_fee)?;
  pool.total_lpt_fee += lpt_amount_fee;

  if !(cheque.borrow_amount > 0) {
    cheque.authority = ctx.accounts.authority.key();
    cheque.pool = pool.key();
    cheque.borrow_amount = 0;
    cheque.base_amount = 0;
  }
  cheque.borrow_amount += lpt_amount_with_fee;

  // Lock lpt token
  token::transfer(
    CpiContext::new(
      ctx.accounts.token_program.to_account_info(),
      token::Transfer {
        from: ctx.accounts.lpt_token_account.to_account_info(),
        to: ctx.accounts.lpt_treasury.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
      },
    ),
    lpt_amount_with_fee,
  )?;
  // Borrow calculate
  let reserves = vec![pool.balance, pool.stable_balance];
  let supply = ctx.accounts.lpt_mint.supply;
  let amounts =
    calc_mint_receives_remove_full_side(lpt_amount_with_fee, supply, &reserves).unwrap();
  let base_amount = amounts[1];
  msg!("Borrow amount {}", base_amount);
  // Update pool info
  pool.base_balance -= base_amount;
  // Pool actions: transfer base mint
  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];
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
    base_amount,
  )?;
  cheque.base_amount += base_amount;
  Ok(())
}
