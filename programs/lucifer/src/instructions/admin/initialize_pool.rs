use crate::constant::*;
use crate::utils::*;
use crate::errors::ErrorCode;
use crate::oracle::*;
use crate::schema::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct InitializePool<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  // Pool's info
  #[account(init, payer = authority, space = Pool::LEN)]
  pub pool: Account<'info, Pool>,
  #[account(seeds = [b"treasurer", &pool.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  // Pool's Mints
  pub mint: Box<Account<'info, token::Mint>>,
  pub base_mint: Box<Account<'info, token::Mint>>,
  #[account(
    init,
    payer = authority,
    mint::decimals = MINT_LPT_DECIMALS,
    mint::authority = treasurer,
    mint::freeze_authority = treasurer,
    seeds = [b"stable_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub stable_mint: Account<'info, token::Mint>,
  #[account(
    init,
    payer = authority,
    mint::decimals = MINT_LPT_DECIMALS,
    mint::authority = treasurer,
    mint::freeze_authority = treasurer,
    seeds = [b"lpt_mint".as_ref(), &pool.key().to_bytes()], bump
  )]
  pub lpt_mint: Account<'info, token::Mint>,
  // Pool's token account
  #[account(
    init,
    payer = authority,
    associated_token::mint = mint,
    associated_token::authority = treasurer
  )]
  pub treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = stable_mint,
    associated_token::authority = treasurer
  )]
  pub stable_treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = base_mint,
    associated_token::authority = treasurer
  )]
  pub base_treasury: Box<Account<'info, token::TokenAccount>>,
  #[account(
    init,
    payer = authority,
    associated_token::mint = lpt_mint,
    associated_token::authority = treasurer
  )]
  pub lpt_treasury: Box<Account<'info, token::TokenAccount>>,
  // Wallet's Token Accounts
  #[account(
    mut,
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
    init,
    payer = authority,
    associated_token::mint = lpt_mint,
    associated_token::authority = authority
  )]
  pub lpt_token_account: Box<Account<'info, token::TokenAccount>>,

  #[account(
    init, 
    payer = authority, 
    space = Cert::LEN, 
    seeds = [&lpt_mint.key().to_bytes(), &authority.key().to_bytes()], bump
  )]
  pub cert: Account<'info, Cert>,

  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(
  ctx: Context<InitializePool>,
  fee: u64,
  amount: u64,
  stable_amount: u64,
  base_amount: u64,
) -> Result<()> {
  let current_time = current_timestamp().unwrap();
  let pool = &mut ctx.accounts.pool;
  let cert = &mut ctx.accounts.cert;
  pool.authority = ctx.accounts.authority.key();
  // Pool's mints
  pool.mint = ctx.accounts.mint.key();
  pool.stable_mint = ctx.accounts.stable_mint.key();
  pool.base_mint = ctx.accounts.base_mint.key();
  pool.lpt_mint = ctx.accounts.lpt_mint.key();

  if !(amount > 0 && stable_amount > 0) {
    return err!(ErrorCode::AmountZero);
  }
  // WALLET ACTIONS
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
  // Transfer Base Mint
  if base_amount > 0 {
    token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.base_token_account.to_account_info(),
          to: ctx.accounts.base_treasury.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
        },
      ),
      base_amount,
    )?;
  }
  // POOL ACTIONS
  // Mint to stable token
  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &pool.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];

  let mint_to_stable = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    token::MintTo {
      to: ctx.accounts.stable_treasury.to_account_info(),
      mint: ctx.accounts.stable_mint.to_account_info(),
      authority: ctx.accounts.treasurer.to_account_info(),
    },
    seeds,
  );
  token::mint_to(mint_to_stable, stable_amount)?;

  let mint_to_stable = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    token::MintTo {
      to: ctx.accounts.stable_token_account.to_account_info(),
      mint: ctx.accounts.stable_mint.to_account_info(),
      authority: ctx.accounts.treasurer.to_account_info(),
    },
    seeds,
  );
  token::mint_to(mint_to_stable, stable_amount)?;
  // Mint to LPT
  let lpt_amount = calc_starting_lpt(amount, stable_amount).unwrap();
  let mint_to_ctx = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    token::MintTo {
      to: ctx.accounts.lpt_treasury.to_account_info(),
      mint: ctx.accounts.lpt_mint.to_account_info(),
      authority: ctx.accounts.treasurer.to_account_info(),
    },
    seeds,
  );
  token::mint_to(mint_to_ctx, lpt_amount)?;
  msg!("starting_lpt_amount {}", lpt_amount);
  // Update pool balance
  pool.balance = amount;
  pool.stable_balance = stable_amount;
  pool.base_balance = base_amount;
  pool.fee = fee;
  pool.total_lpt_fee = fee;
  pool.lpt_supply = lpt_amount ;
  pool.start_time = current_time;
  // Update Cert
  cert.authority = ctx.accounts.authority.key();
  cert.pool = pool.key();
  cert.amount = calc_starting_lpt(amount, base_amount).unwrap();
  msg!("cert.amount {}", cert.amount);
  Ok(())
}
