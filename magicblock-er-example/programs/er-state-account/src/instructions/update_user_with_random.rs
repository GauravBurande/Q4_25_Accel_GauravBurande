use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

use crate::state::UserAccount;
use crate::{instruction, ID};

#[vrf]
#[derive(Accounts)]
pub struct VrfRandomBase<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"user", user.key().as_ref()],
        bump= user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: The oracle queue
    #[account(
        mut,
        address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE
    )]
    pub oracle_queue: AccountInfo<'info>,
}

#[vrf]
#[derive(Accounts)]
pub struct VrfRandomEr<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"user", user.key().as_ref()],
        bump= user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: The oracle queue
    #[account(
        mut,
        address = ephemeral_vrf_sdk::consts::DEFAULT_EPHEMERAL_QUEUE
    )]
    pub oracle_queue: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct VrfCallback<'info> {
    #[account(address=ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl VrfRandomBase<'_> {
    pub fn vrf_random_base(&mut self, client_seed: u8) -> Result<()> {
        msg!("Requesting a random number!");
        let ixn = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: instruction::VrfCallback::DISCRIMINATOR.to_vec(),
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            caller_seed: [client_seed; 32],
            ..Default::default()
        });

        self.invoke_signed_vrf(&self.user.to_account_info(), &ixn)?;

        Ok(())
    }
}

impl VrfRandomEr<'_> {
    pub fn vrf_random_er(&mut self, client_seed: u8) -> Result<()> {
        msg!("Requesting a random number!");
        let ixn = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: instruction::VrfCallback::DISCRIMINATOR.to_vec(),
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            caller_seed: [client_seed; 32],
            ..Default::default()
        });

        self.invoke_signed_vrf(&self.user.to_account_info(), &ixn)?;

        Ok(())
    }
}

impl VrfCallback<'_> {
    pub fn update_user(&mut self, rnd_u8: u8) -> Result<()> {
        self.user_account.data = rnd_u8 as u64;

        Ok(())
    }
}
