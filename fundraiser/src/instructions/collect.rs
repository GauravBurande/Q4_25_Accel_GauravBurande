use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{constant::SECONDS_TO_DAYS, instructions::validate_ata, state::Fundraiser};

pub fn process_collect(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, maker_ata, mint, fundraiser, vault, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    if !maker.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    validate_ata(maker_ata, mint, maker)?;
    validate_ata(vault, mint, fundraiser)?;

    let fundraiser_data = fundraiser.try_borrow_data()?;
    let fundraiser_state = bytemuck::pod_read_unaligned::<Fundraiser>(&fundraiser_data);

    // check if target amount is met or more
    if fundraiser_state.current_amount() < fundraiser_state.amount_to_raise() {
        return Err(pinocchio::program_error::ProgramError::InvalidArgument);
    }
    // check if duration passed
    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time
        < fundraiser_state.time_started() + (fundraiser_state.duration() as u64 * SECONDS_TO_DAYS)
    {
        return Err(pinocchio::program_error::ProgramError::InvalidArgument);
    }

    let bump = data[0];
    Fundraiser::validate_pda(bump, fundraiser.key(), maker.key())?;

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);
    let amount = fundraiser_state.current_amount();
    Transfer {
        from: vault,
        to: maker_ata,
        amount,
        authority: fundraiser,
    }
    .invoke_signed(&[seeds])?;
    Ok(())
}
