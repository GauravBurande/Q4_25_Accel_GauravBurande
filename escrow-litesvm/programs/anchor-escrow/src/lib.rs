#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod error;
mod instructions;
mod state;
mod tests;

use instructions::*;

declare_id!("3FDewnyxSEbLXYZVJ64rz5iFm1HPTpR856qQnFuh29KM");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        deposit: u64,
        receive: u64,
        freeze_period: u32,
    ) -> Result<()> {
        ctx.accounts
            .init_escrow(seed, receive, freeze_period, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }
}
