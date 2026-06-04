use crate::Fundraiser;
use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::Transfer;

pub fn process_checker_instruction(accounts: &mut [AccountView]) -> ProgramResult {
    let [
        maker,
        fundraiser_account,
        vault,
        maker_ata,
        _token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::InvalidAccountData);
    };

    let fundraiser_state_pda = Fundraiser::from_account_info(fundraiser_account)?;

    let fundraiser_bump = [fundraiser_state_pda.bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&fundraiser_bump),
    ];

    let transfer_amount = u64::from_le_bytes(fundraiser_state_pda.current_amount);

    Transfer::new(vault, maker_ata, fundraiser_account, transfer_amount)
        .invoke_signed(&[Signer::from(&signer_seeds)])?;
    Ok(())
}
