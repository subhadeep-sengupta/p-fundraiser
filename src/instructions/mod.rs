pub mod initialize;

pub use initialize::*;
use pinocchio::error::ProgramError;

pub enum FundraiserInstructions {
    Initialize = 0,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
