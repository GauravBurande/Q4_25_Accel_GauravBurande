use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_cancel_instruction(accounts: &[AccountInfo]) -> ProgramResult {
    let [maker, mint_a, escrow_account, maker_ata_a, escrow_ata_a, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    //checks:
    // make sure there is a signer
    if !maker.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }
    // make sure the maker is owner of the ata
    {
        let maker_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(maker_ata_a)?;

        if maker.key() != maker_ata_a_state.owner() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }

        // check if the mint is correct
        if mint_a.key() != maker_ata_a_state.mint() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }
    {
        let escrow_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_info(escrow_ata_a)?;
        if escrow_ata_a_state.owner() != escrow_account.key() {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
        if escrow_ata_a_state.mint() != mint_a.key() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    if escrow_account.owner() != &crate::ID {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }
    // read the escrow pda to get amount
    let escrow_state = Escrow::from_account_info(escrow_account)?;

    let bump = escrow_state.bump;
    let seed = [b"escrow", maker.key().as_slice(), &[bump]];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID);
    assert_eq!(&escrow_account_pda, escrow_account.key());

    if escrow_state.maker() != *maker.key() {
        return Err(pinocchio::program_error::ProgramError::IllegalOwner);
    }

    let amount_to_give = escrow_state.amount_to_give();

    // send the tokens from vault to maker_ata
    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        amount: amount_to_give,
        from: escrow_ata_a,
        to: maker_ata_a,
        authority: escrow_account,
    }
    .invoke_signed(&[seeds])?;

    let close_seeds = Signer::from(&seed);
    // close vault
    // do this first because we gonna borrow mutate after this
    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata_a,
        authority: escrow_account,
        destination: maker,
    }
    .invoke_signed(&[close_seeds])?;

    // close escrow
    let lamports_at_escrow = escrow_account.lamports();

    let mut maker_lamports = maker.try_borrow_mut_lamports()?;
    *maker_lamports += lamports_at_escrow;

    {
        let mut escrow_acc_lamports = escrow_account.try_borrow_mut_lamports()?;
        *escrow_acc_lamports -= lamports_at_escrow;
    }

    escrow_account.close()?;

    Ok(())
}
