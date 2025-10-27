use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_pubkey::derive_address;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Contributor {
    pub amount: [u8; 8],
}

impl Contributor {
    pub const LEN: usize = core::mem::size_of::<Contributor>();

    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;

        if data.len() != Contributor::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seeds = [b"contributor".as_ref(), owner.as_slice()];
        let derived = derive_address(&seeds, Some(bump), &crate::ID);

        if derived != *pda {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }
}
