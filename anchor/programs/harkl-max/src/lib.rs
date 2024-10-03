use anchor_lang::prelude::*;

declare_id!("5m94oNVhSyFesxHorRsDCAv7VYYxAcftmDc1sAQa8JPh");

#[program]
pub mod harkl_max {
    use super::*;

    pub fn greet(_ctx: Context<Initialize>) -> Result<()> {
        msg!("GM!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
