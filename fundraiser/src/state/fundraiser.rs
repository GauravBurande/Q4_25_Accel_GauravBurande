use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_pubkey::derive_address;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 1], // in days
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = core::mem::size_of::<Fundraiser>();

    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut_data()?;

        if data.len() != Fundraiser::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn validate_pda(bump: u8, pda: &Pubkey, maker: &Pubkey) -> Result<(), ProgramError> {
        let seeds = &[b"fundraiser".as_ref(), maker.as_slice()];
        let derived = derive_address(seeds, Some(bump), &crate::ID);

        if derived != *pda {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn maker(&self) -> Pubkey {
        Pubkey::from(self.maker)
    }

    pub fn set_maker(&mut self, maker: &Pubkey) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn mint_to_raise(&self) -> Pubkey {
        Pubkey::from(self.mint_to_raise)
    }

    pub fn set_mint_to_raise(&mut self, mint_to_raise: &Pubkey) {
        self.mint_to_raise.copy_from_slice(mint_to_raise.as_ref());
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount_to_raise: u64) {
        self.amount_to_raise = amount_to_raise.to_le_bytes();
    }

    pub fn current_amount(&self) -> u64 {
        u64::from_le_bytes(self.current_amount)
    }

    pub fn set_current_amount(&mut self, current_amount: u64) {
        self.current_amount = current_amount.to_le_bytes();
    }

    pub fn time_started(&self) -> u64 {
        u64::from_le_bytes(self.time_started)
    }

    pub fn set_time_started(&mut self, time_started: u64) {
        self.time_started = time_started.to_le_bytes();
    }
    pub fn duration(&self) -> u8 {
        self.duration[0]
    }

    pub fn set_duration(&mut self, duration: u8) {
        self.duration[0] = duration;
    }
    pub fn bump(&self) -> u8 {
        self.bump
    }

    pub fn set_bump(&mut self, bump: u8) {
        self.bump = bump;
    }
}
