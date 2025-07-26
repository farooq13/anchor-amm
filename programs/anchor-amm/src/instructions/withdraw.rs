use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, {token::{transfer, Burn, Mint, Token, TokenAccount}}};
use constant_product_curve::ConstantProduct;

use crate::{errors::AmmError, state::Config};


#[derive(Accounts)]
pub struct Withdraw<'info> {
  #[account(mut)]
  pub user: Signer<'info>,
  pub mint_x: Account<'info, Mint>,
  pub mint_y: Account<'info, Mint>,

  #[account(
    has_one = mint_x,
    has_one = mint_y,
    seeds = [b"config", config.seed.to_le_bytes().as_ref()],
    bump = config.config_bump
  )]
  pub config: Account<'info, Config>,

  #[account(
    mut,
    seeds = [b"lp", config.key().as_ref()],
    bump = config.lp_bump,
  )]
  pub mint_lp: Account<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = config,
  )]
  pub vault_x: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = config,
  )]
  pub vault_y: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = mint_x,
    associated_token::authority = user,
  )]
  pub user_x: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = mint_y,
    associated_token::authority = user,
  )]
  pub user_y: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_lp,
    associated_token::authority = config,
  )]
  pub user_lp: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,

}

impl<'info> Withdraw<'info> {
  pub fn withdraw(&mut self, amount: u64, mint_x: u64, mint_y: u64) -> Result<()> {
    require!(self.config.locked == false, AmmError::PoolLocked);
    require!(amount != 0, AmmError::InvalidAmount);
    require!(mint_x != 0 || mint_y != 0, AmmError::InvalidAmount);

    let amounts = ConstantProduct::xy_withdraw_amounts_from(
      self.vault_x.amount,
      self.vault_y.amount,
      self.mint_lp.supply,
      amount,
      6,
    )
      .map_err(AmmError::from)?;

    require!(mint_x <= amounts.x && mint_y <= amounts.y, AmmError::SlippageExceeded);

    self.new_with_signer(true, amounts.x)?;
    self.withdraw_tokens(false, amounts.y)?;
    self.burn_lp_tokens(amount)?;

    Ok(())
  }

  pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
    let (from, to) = match is_x {
      true => (self.vault_x.to_account_info(), self.user_x.to_account_info()),
      false => (self.vault_y.to_account_info(), self.user_y.to_account_info()),
    };

    let cpi_program = self.token_program.to_account_info();
    
    let accounts = Transfer {
      from,
      to,
      authority: self.config.to_account_info(),
    };

    let seeds = &[
      &b"congi"[..],
      &self.config.seed.to_le_bytes(),
      &[self.config.config_bump],
    ];

    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(cpi_program, account, signer_seeds);

    transfer(cpi_context, amount)?;
  }
}