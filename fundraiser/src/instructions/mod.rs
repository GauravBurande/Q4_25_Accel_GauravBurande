use pinocchio::program_error::ProgramError;

pub mod collect;
pub mod contribute;
pub mod initialize;
pub mod refund;

pub use collect::*;
pub use contribute::*;
pub use initialize::*;
pub use refund::*;

pub enum FundInstructions {
    Initialize = 0,
    Contribute = 1,
    Refund = 2,
    Collect = 3,
}

impl TryFrom<&u8> for FundInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundInstructions::Initialize),
            1 => Ok(FundInstructions::Contribute),
            2 => Ok(FundInstructions::Refund),
            3 => Ok(FundInstructions::Collect),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
