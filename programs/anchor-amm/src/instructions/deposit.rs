use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Transfer, transfer, Mint, Token, TokenAccount}};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config};


#[derive(Accounts)]
