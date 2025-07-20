use anchor_lang::prelude::*;

declare_id!("6KEWdkExSDEjo64bNfVAX82qRdCG2eLnpg2BkyLTHChP");

#[program]
pub mod anchor_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
