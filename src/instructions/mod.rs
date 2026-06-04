pub mod checker;
pub mod contribute;
pub mod create_contributor;
pub mod initialize;
pub mod refund;

pub use checker::*;
pub use contribute::*;
pub use create_contributor::*;
pub use initialize::*;
use pinocchio::error::ProgramError;
pub use refund::*;

pub enum FundraiserInstructions {
    Initialize = 0,
    CreateContributor = 1,
    Contribute = 2,
    Checker = 3,
    Refund = 4,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::CreateContributor),
            2 => Ok(FundraiserInstructions::Contribute),
            3 => Ok(FundraiserInstructions::Checker),
            4 => Ok(FundraiserInstructions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
