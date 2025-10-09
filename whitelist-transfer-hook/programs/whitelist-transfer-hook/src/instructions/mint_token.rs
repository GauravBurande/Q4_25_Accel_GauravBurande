use anchor_lang::{ 
    prelude::*, 
};
use anchor_spl::{associated_token::{get_associated_token_address, AssociatedToken}, token_2022::{mint_to_checked, MintToChecked}, token_interface::{
    Mint, TokenAccount, TokenInterface
}};

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub source_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(mut)]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [b"whitelist", user.key().as_ref()], 
        bump=whitelist.bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken> 
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, _bumps: &TokenFactoryBumps) -> Result<()> {
        msg!("Initializing Mint...");

        // let token_ata = get_associated_token_address(&self.user.key(), &self.mint.key());

        let cpi_program = self.token_program.to_account_info();
        let amount = 10_u64.pow(self.mint.decimals.into());

        let cpi_ctx = CpiContext::new(cpi_program, MintToChecked {
            authority: self.user.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.source_token_account.to_account_info()
        });

        mint_to_checked(cpi_ctx, amount, self.mint.decimals)?;
        Ok(())
    }
}