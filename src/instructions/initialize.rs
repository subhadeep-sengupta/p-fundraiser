use pinocchio::{
    AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::Fundraiser;

pub fn process_initialize_instruction(
    accounts: &mut [AccountView],
    data: &[u8],
) -> Result<(), ProgramError> {
    let [
        maker,
        mint_to_raise,
        fundraiser_account,
        vault,
        system_program,
        token_program,
        _associated_token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let bump = data[0];

    let amount_to_raise = u64::from_le_bytes(
        data[1..9]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    let seeds: [&[u8]; 3] = [b"fundraiser", maker.address().as_ref(), &[bump]];

    let fundraiser_account_pda = derive_address(&seeds, None, &crate::ID.to_bytes());

    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_bytes = [bump];

    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(bump_bytes.as_ref()),
    ];

    CreateAccount {
        from: maker,
        to: fundraiser_account,
        owner: &crate::ID,
        space: Fundraiser::LEN as u64,
        lamports: Rent::get()?.try_minimum_balance(Fundraiser::LEN)?,
    }
    .invoke_signed(&[Signer::from(&signer_seeds)])?;

    let fundraiser_state = Fundraiser::from_account_info(fundraiser_account)?;

    fundraiser_state.maker = *maker.address().as_array();
    fundraiser_state.mint_to_raise = *mint_to_raise.address().as_array();
    fundraiser_state.amount_to_raise = amount_to_raise.to_le_bytes();
    fundraiser_state.time_started = Clock::get()?.unix_timestamp.to_le_bytes().map(|b| b as i8);
    fundraiser_state.duration = data[9];
    fundraiser_state.current_amount = 0u64.to_le_bytes();

    Create {
        funding_account: maker,
        token_program,
        system_program,
        mint: mint_to_raise,
        wallet: fundraiser_account,
        account: vault,
    }
    .invoke()?;

    Ok(())
}
