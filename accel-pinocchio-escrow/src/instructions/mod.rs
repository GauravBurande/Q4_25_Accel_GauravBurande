pub mod cancel;
pub mod make;

pub use cancel::*;
pub use make::*;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Cancel = 2,
    MakeV2 = 3,
}

impl TryFrom<&u8> for EscrowInstrctions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstrctions::Make),
            1 => Ok(EscrowInstrctions::Take),
            2 => Ok(EscrowInstrctions::Cancel),
            3 => Ok(EscrowInstrctions::MakeV2),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
