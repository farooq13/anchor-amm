use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, Transfer}};
use constant_product_curve::{ContantProduct, LiqudityPair};

use crate::{errors::AmmError, state::Config};


#[derive(Accounts)]
pub struct Swap<'info> {
  #[account(mut)]
  pub user: Signer<'info>,
  pub mint_x: Account<'info, Mint>,
  pub mint_y: Account<'info, Mint>,


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
    has_one = mint_x,
    has_one = mint_y,
    seeds = [b"config", config.seed.to_le_bytes().as_reef()],
    bump = config.config_bump,
  )]
  pub config: Account<'info, Config>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,

}

impl<'info> Swap<'info> {
  pub fn swap(&mut self, is_x: bool, amount: u64) -> Result<()> {
    require!(self.config.locked == false, AmmError::PoolLocked);
    require!(amount > 0, AmmError::InvalidAmount);

    let mut curve = ConstantProduct::init(
      self.vault_x.amount,
      self.vault_y.amount,
      self.vault_x.amount,
      self.config.fee,
      None,
    )
    .map_err(AmmError::from)?;

    let p = match is_x {
      true => LiquidityPair::X,
      false => LigidityPair::Y,
    };

    let res = curve.swap(p, amount, min).map_err(AmmError::from)?;

    require!(res.deposit != 0, AmmError::InvalidAmount);
    require!(res.withdraw != 0, AmmError::InvalidAmount);

    // deposit tokens
    self.deposit_tokens(is_x, res.deposit)?;
    
    // withdraw tokens
    self.withdraw_tokens(is_x, res.withdraw)?;

    Ok(())
  }
}