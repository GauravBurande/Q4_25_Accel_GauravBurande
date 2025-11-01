use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    instructions::validate_ata,
    state::{Contributor, Fundraiser},
};

pub fn process_refund(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [contributor, contributor_ata, contributor_pda, mint, fundraiser, vault, _system_program, _token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
    };

    if !contributor.is_signer() {
        return Err(pinocchio::program_error::ProgramError::MissingRequiredSignature);
    }

    let fundraiser_data = fundraiser.try_borrow_data()?;
    let fundraiser_state = bytemuck::try_pod_read_unaligned::<Fundraiser>(&fundraiser_data)
        .expect("Invalid fundraiser data");

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

    // refund the contributor
    let maker: Pubkey = fundraiser_state.maker();
    let bump = [fundraiser_state.bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(&maker),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);

    {
        let contributor_data = contributor_pda.try_borrow_data()?;
        let contributor_state = bytemuck::try_pod_read_unaligned::<Contributor>(&contributor_data)
            .expect("Invalid contributor data");

        Transfer {
            from: vault,
            to: contributor_ata,
            amount: contributor_state.amount(),
            authority: fundraiser,
        }
        .invoke_signed(&[seeds])?;
    }

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
