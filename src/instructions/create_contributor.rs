use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_system::instructions::CreateAccount;

use crate::Contributor;

pub fn process_create_contributor_instruction(
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let [
        contributor_account,
        fundraiser_account,
        contributor_state_account,
        _system_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::InvalidAccountData);
    };

    let bump = data[0];
    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser_account.address().as_array()),
        Seed::from(contributor_account.address().as_array()),
        Seed::from(&bump_bytes),
    ];

    CreateAccount {
        owner: &crate::ID,
        from: contributor_account,
        to: contributor_state_account,
        lamports: Rent::get()?.try_minimum_balance(Contributor::LEN)?,
        space: Contributor::LEN as u64,
    }
    .invoke_signed(&[Signer::from(&signer_seeds)])?;

    let contributor_state_data = unsafe { contributor_state_account.borrow_unchecked_mut() };
    let contributor_state =
        unsafe { &mut *(contributor_state_data.as_mut_ptr() as *mut Contributor) };
    contributor_state.address = *contributor_state_account.address().as_array();
    contributor_state.amount = 0u64.to_le_bytes();
    contributor_state.bump = bump;
    Ok(())
}
