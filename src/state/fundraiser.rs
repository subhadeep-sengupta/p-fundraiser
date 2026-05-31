use pinocchio::{AccountView, error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [i8; 8],
    pub duration: u8,
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1;

    pub fn from_account_info(account_info: &mut AccountView) -> Result<&mut Self, ProgramError> {
        let data = unsafe { account_info.borrow_unchecked_mut() };
        if data.len() != Fundraiser::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}

// pub struct Fundraiser {
//     pub maker: Pubkey,
//     pub mint_to_raise: Pubkey,
//     pub amount_to_raise: u64,
//     pub current_amount: u64,
//     pub time_started: i64,
//     pub duration: u8,
//     pub bump: u8,
// }
