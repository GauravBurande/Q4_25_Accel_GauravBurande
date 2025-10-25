use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    ProgramResult,
};
use pinocchio_pubkey::derive_address;

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountInfo]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, escrow, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _associated_token_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    // checks:

    // make sure taker is a signer
    if !taker.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    // make sure maker is owner of the ata
    {
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_info(maker_ata_b)?;
        if maker.key() != maker_ata_state.owner() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountOwner);
        }

        if mint_b.key() != maker_ata_state.mint() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }
    // make sure taker is owner of the taker_ata_b, don't check for taker_ata_a, u must be dumb enough to send someone else's ata account
    {
        let taker_ata_state = pinocchio_token::state::TokenAccount::from_account_info(taker_ata_b)?;
        if taker.key() != taker_ata_state.owner() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountOwner);
        }

        if mint_b.key() != taker_ata_state.mint() {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    // read escrow pda for the amounts and other config
    let escrow_state = Escrow::from_account_info(escrow)?;
    // check the maker address is right
    if *maker.key() != escrow_state.maker() {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    }

    let bump = escrow_state.bump;
    let seeds = [b"escrow", maker.key().as_slice(), &[bump]];
    let escrow_pda = derive_address(&seeds, None, &crate::ID);

    assert_eq!(&escrow_pda, escrow.key());

    // but make sure maker_ata_b and taker_ata_a exists
    pinocchio_associated_token_account::instructions::CreateIdempotent {
        account: maker_ata_b,
        funding_account: taker,
        mint: mint_b,
        wallet: maker,
        system_program,
        token_program,
    }
    .invoke()?;

    pinocchio_associated_token_account::instructions::CreateIdempotent {
        account: taker_ata_a,
        funding_account: taker,
        mint: mint_a,
        wallet: taker,
        system_program,
        token_program,
    }
    .invoke()?;

    // transfer token from taker_ata_b to maker_ata_b
    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        amount: escrow_state.amount_to_receive(),
        authority: taker,
    }
    .invoke()?;

    // transfer token from vault to taker_ata_a
    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.key()),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);
    let close_vault_seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: vault,
        to: taker_ata_a,
        amount: escrow_state.amount_to_give(),
        authority: escrow,
    }
    .invoke_signed(&[seeds])?;

    // close vault
    pinocchio_token::instructions::CloseAccount {
        account: vault,
        destination: maker,
        authority: escrow,
    }
    .invoke_signed(&[close_vault_seeds])?;

    // close escrow
    let lamports = escrow.lamports();
    let mut maker_lamports = maker.try_borrow_mut_lamports()?;
    *maker_lamports += lamports;
    {
        let mut escrow_lamports = escrow.try_borrow_mut_lamports()?;
        *escrow_lamports -= lamports;
    };

    escrow.close()?;

    Ok(())
}
