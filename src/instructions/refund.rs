use crate::{Contributor, Fundraiser};
use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::Transfer;

pub fn process_refund_instruction(accounts: &mut [AccountView]) -> ProgramResult {
    let [
        contributor_account,
        vault,
        fundraiser_account,
        contributor_pda_account,
        contributor_ata,
        _token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::InvalidAccountData);
    };

    let fundraiser_state_pda = Fundraiser::from_account_info(fundraiser_account)?;
    let maker = fundraiser_state_pda.maker;
    let fundraiser_bump = [fundraiser_state_pda.bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(&maker),
        Seed::from(&fundraiser_bump),
    ];

    let contributor_state_pda = Contributor::from_account_info(contributor_pda_account)?;
    let refund_amount = u64::from_le_bytes(contributor_state_pda.amount);

    Transfer::new(vault, contributor_ata, fundraiser_account, refund_amount)
        .invoke_signed(&[Signer::from(&signer_seeds)])?;

    contributor_account
        .set_lamports(contributor_pda_account.lamports() + contributor_account.lamports());
    contributor_pda_account.set_lamports(0);
    contributor_pda_account.close()?;
    Ok(())
}
