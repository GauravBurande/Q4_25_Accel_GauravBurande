use pinocchio::{account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey, ProgramResult};

use crate::instructions::FundInstructions;

mod constant;
mod instructions;
mod state;
mod tests;

pinocchio_pubkey::declare_id!("BbFoDc7zsPk4QJLQmL6boWhc4HoGWbW8w4PPXGbdNfKL");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("yo! ser, hyd?");
    assert_eq!(program_id, &crate::ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)?;

    match FundInstructions::try_from(discriminator)? {
        FundInstructions::Initialize => instructions::process_initialize(accounts, data)?,
        _ => return Err(pinocchio::program_error::ProgramError::InvalidAccountData),
    }
    Ok(())
}
