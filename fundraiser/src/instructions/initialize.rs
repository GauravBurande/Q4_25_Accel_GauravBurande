use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_system::instructions::CreateAccount;

use crate::{constant::MIN_AMOUNT_TO_RAISE, state::Fundraiser};

pub fn process_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [maker, mint, fundraiser, vault, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    // checks:
    if !maker.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let bump = data[0];

    Fundraiser::validate_pda(bump, fundraiser.key(), maker.key())?;

    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
    let duration = data[9];

    let mint_state = pinocchio_token::state::Mint::from_account_info(mint)?;

    if amount < MIN_AMOUNT_TO_RAISE * 10u64.pow(mint_state.decimals() as u32) {
        return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
    }

    if fundraiser.owner() != &crate::ID {
        let bump = [bump.to_le()];
        let seed = [
            Seed::from(b"fundraiser"),
            Seed::from(maker.key()),
            Seed::from(&bump),
        ];

        let seeds = Signer::from(&seed);
        CreateAccount {
            from: maker,
            to: fundraiser,
            owner: &crate::ID,
            space: Fundraiser::LEN as u64,
            lamports: Rent::get()?.minimum_balance(Fundraiser::LEN),
        }
        .invoke_signed(&[seeds])?;

        {
            let mut account_data = fundraiser.try_borrow_mut_data()?;
            let fundraiser_state = bytemuck::try_from_bytes_mut::<Fundraiser>(&mut account_data)
                .map_err(|_| pinocchio::program_error::ProgramError::InvalidInstructionData)?;

            fundraiser_state.set_maker(maker.key());
            fundraiser_state.set_mint_to_raise(mint.key());
            fundraiser_state.set_amount_to_raise(amount);
            fundraiser_state.set_current_amount(0);
            fundraiser_state.set_time_started(Clock::get()?.unix_timestamp as u64);
            fundraiser_state.set_duration(duration);
            fundraiser_state.bump = data[0];
        }
    } else {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    CreateIdempotent {
        funding_account: maker,
        account: vault,
        wallet: fundraiser,
        mint,
        system_program,
        token_program,
    }
    .invoke()?;

    Ok(())
}
