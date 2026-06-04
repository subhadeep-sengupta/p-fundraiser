#![allow(unexpected_cfgs)]

use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, entrypoint, error::ProgramError,
};

pub mod constants;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

entrypoint!(process_instruction);

declare_id!("GrwpvrmM3qhe3meXTD32LTLY3ggJMHPoe7vcEiE8dr6f");

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => {
            instructions::process_initialize_instruction(accounts, data)?
        }
        FundraiserInstructions::CreateContributor => {
            instructions::process_create_contributor_instruction(accounts, data)?
        }
        FundraiserInstructions::Contribute => {
            instructions::process_contribute_instruction(accounts, data)?
        }
        FundraiserInstructions::Checker => instructions::process_checker_instruction(accounts)?,
        FundraiserInstructions::Refund => instructions::process_refund_instruction(accounts)?,
    }

    Ok(())
}
