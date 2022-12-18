use crate::schema::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

// Only Test Devnet
// Wait implement Jupiter Mainnet

#[derive(Accounts)]
pub struct SwapJupiter<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  #[account(mut)]
  pub jupiter: Account<'info, Jupiter>,
  //
  #[account(seeds = [b"treasurer", &jupiter.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  // Pool's Mints
  #[account(
    mut,
    seeds = [b"base_mint".as_ref(), &jupiter.key().to_bytes()], bump
  )]
  pub base_mint: Account<'info, token::Mint>,
  pub mint: Account<'info, token::Mint>,

  #[account(
    init_if_needed,
    payer=authority,
    associated_token::mint = mint,
    associated_token::authority = treasurer
  )]
  pub mint_treasury: Box<Account<'info, token::TokenAccount>>,

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
    associated_token::mint = base_mint,
    associated_token::authority = authority
  )]
  pub base_token_account: Box<Account<'info, token::TokenAccount>>,

  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<SwapJupiter>, amount_in: u64, base_amount: u64) -> Result<()> {
  let jupiter = &mut ctx.accounts.jupiter;

  let seeds: &[&[&[u8]]] = &[&[
    "treasurer".as_ref(),
    &jupiter.key().to_bytes(),
    &[*ctx.bumps.get("treasurer").unwrap()],
  ]];

  if amount_in > 0 {
    token::transfer(
      CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
          from: ctx.accounts.token_account.to_account_info(),
          to: ctx.accounts.mint_treasury.to_account_info(),
          authority: ctx.accounts.authority.to_account_info(),
        },
      ),
      amount_in,
    )?;
  }

  let mint_to_stable = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    token::MintTo {
      to: ctx.accounts.base_token_account.to_account_info(),
      mint: ctx.accounts.base_mint.to_account_info(),
      authority: ctx.accounts.treasurer.to_account_info(),
    },
    seeds,
  );
  token::mint_to(mint_to_stable, base_amount)?;

  Ok(())
}
