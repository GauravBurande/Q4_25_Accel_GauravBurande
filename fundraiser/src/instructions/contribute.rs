use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{
    instructions::Transfer,
    state::{Mint, TokenAccount},
};

use crate::{
    constant::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
    state::{Contributor, Fundraiser},
};

pub fn process_contribute(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, contributor_ata, contributor_pda, mint, fundraiser, vault, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    // checks:
    if !contributor.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;
    if fundraiser_state.mint_to_raise.is_empty() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }
    Fundraiser::validate_pda(
        fundraiser_state.bump(),
        &fundraiser.key(),
        &fundraiser_state.maker(),
    )?;

    let bump = data[0];

    Contributor::validate_pda(bump, &contributor_pda.key(), &contributor.key())?;

    {
        let contributor_ata_state = TokenAccount::from_account_info(contributor_ata)?;
        if mint.key() != contributor_ata_state.mint() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        if contributor_ata_state.owner() != contributor.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    {
        let vault_state = TokenAccount::from_account_info(vault)?;
        if mint.key() != vault_state.mint() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        if vault_state.owner() != fundraiser.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    let mint_state = Mint::from_account_info(mint)?;

    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());

    if amount < 1_u8.pow(mint_state.decimals() as u32) as u64 {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
    }

    if amount
        >= (fundraiser_state.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER
    {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
    }

    let current_time = Clock::get()?.unix_timestamp as u64;

    if current_time
        >= fundraiser_state.time_started() + (fundraiser_state.duration() as u64 * SECONDS_TO_DAYS)
    {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"contributor"),
        Seed::from(contributor.key()),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);

    if contributor.owner() != &crate::ID {
        CreateAccount {
            from: contributor,
            to: contributor_pda,
            owner: &crate::ID,
            space: Contributor::LEN as u64,
            lamports: Rent::get()?.minimum_balance(Contributor::LEN),
        }
        .invoke_signed(&[seeds])?;

        {
            let contributor_state = Contributor::from_account_info(contributor_pda)?;
            contributor_state.set_amount(amount);
        }
    } else {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let contributor_state = Contributor::from_account_info(contributor_pda)?;

    if (contributor_state.amount()
        >= (fundraiser_state.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER)
        && (contributor_state.amount() + amount
            >= (fundraiser_state.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE)
                / PERCENTAGE_SCALER)
    {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
    }

    // transfer from contributor_ata to vault
    Transfer {
        from: contributor_ata,
        to: vault,
        amount: amount,
        authority: contributor,
    }
    .invoke()?;

    fundraiser_state.set_current_amount(
        fundraiser_state
            .current_amount()
            .checked_add(amount)
            .expect("Failed to add amount to current amount!"),
    );

    contributor_state.set_amount(
        contributor_state
            .amount()
            .checked_add(amount)
            .expect("Failed to update amount in contributor PDA!"),
    );
    Ok(())
}
