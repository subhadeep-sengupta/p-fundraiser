use pinocchio::{AccountView, error::ProgramError};
use pinocchio_token::instructions::Transfer;

use crate::Contributor;

pub fn process_contribute_instruction(
    accounts: &mut [AccountView],
    data: &[u8],
) -> Result<(), ProgramError> {
    let [
        contributor_account,
        fundraiser_pda,
        vault,
        contributor_ata,
        contributor_state_pda,
        token_program,
    ] = accounts
    else {
        return Err(ProgramError::InvalidAccountData);
    };

    if !contributor_account.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());

    Transfer::new(contributor_ata, vault, contributor_account, amount).invoke()?;
    let contributor_state = Contributor::from_account_info(contributor_state_pda)?;

    let current_amount = u64::from_le_bytes(contributor_state.amount);
    let new_amount = current_amount
        .checked_add(amount)
        .ok_or(ProgramError::InvalidInstructionData)?;

    contributor_state.amount = new_amount.to_le_bytes();
    Ok(())
}
