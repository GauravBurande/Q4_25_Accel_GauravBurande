use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::Mint};

use crate::{
    constant::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
    instructions::validate_ata,
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

    let mut fundraiser_data = fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = bytemuck::from_bytes_mut::<Fundraiser>(&mut fundraiser_data);
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

    validate_ata(contributor_ata, mint, contributor)?;
    validate_ata(vault, mint, fundraiser)?;

    let mint_state = Mint::from_account_info(mint)?;

    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());

    if amount < 1 * 10u8.pow(mint_state.decimals() as u32) as u64 {
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
    } else {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let mut contributor_data = contributor_pda.try_borrow_mut_data()?;
    let contributor_state = bytemuck::from_bytes_mut::<Contributor>(&mut contributor_data);

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
