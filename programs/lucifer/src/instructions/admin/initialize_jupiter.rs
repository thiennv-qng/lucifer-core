use crate::constant::*;
use crate::schema::*;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

// Test for Devnet
#[derive(Accounts)]
pub struct InitializeJupiter<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  #[account(init, payer = authority, space = Jupiter::LEN)]
  pub jupiter: Account<'info, Jupiter>,
  //
  #[account(seeds = [b"treasurer", &jupiter.key().to_bytes()], bump)]
  /// CHECK: Just a pure account
  pub treasurer: AccountInfo<'info>,
  ///
  // Pool's Mints
  #[account(
    init,
    payer = authority,
    mint::decimals = MINT_LPT_DECIMALS,
    mint::authority = treasurer,
    mint::freeze_authority = treasurer,
    seeds = [b"base_mint".as_ref(), &jupiter.key().to_bytes()], bump
  )]
  pub base_mint: Account<'info, token::Mint>,

  // programs
  pub system_program: Program<'info, System>,
  pub token_program: Program<'info, token::Token>,
  pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
  pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<InitializeJupiter>) -> Result<()> {
  let jupiter = &mut ctx.accounts.jupiter;
  jupiter.base_mint = ctx.accounts.base_mint.key();
  Ok(())
}
