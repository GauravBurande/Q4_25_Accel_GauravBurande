use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::state::{Contributor, Fundraiser};

pub fn process_refund(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, contributor_ata, contributor_pda, mint, fundraiser, vault, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    if !contributor.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let fundraiser_state = Fundraiser::from_account_info(fundraiser)?;
    let contributor_state = Contributor::from_account_info(contributor_pda)?;
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

    // refund the contributor
    let maker: Pubkey = fundraiser_state.maker();
    let bump = [fundraiser_state.bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(&maker),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);

    Transfer {
        from: vault,
        to: contributor_ata,
        amount: contributor_state.amount(),
        authority: fundraiser,
    }
    .invoke_signed(&[seeds])?;

    // close the contributor pda
    let lamports = contributor_pda.lamports();
    let mut contributor_lamports = contributor.try_borrow_mut_lamports()?;
    *contributor_lamports += lamports;

    {
        let mut contributor_pda_lamports = contributor_pda.try_borrow_mut_lamports()?;
        *contributor_pda_lamports -= lamports;
    }

    contributor_pda.close()?;

    Ok(())
}
