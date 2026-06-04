use pinocchio::{AccountView, error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Contributor {
    pub address: [u8; 32],
    pub amount: [u8; 8],
    pub bump: u8,
}

impl Contributor {
    pub const LEN: usize = core::mem::size_of::<Self>();

    pub fn from_account_info(account_info: &mut AccountView) -> Result<&mut Self, ProgramError> {
        let data = unsafe { account_info.borrow_unchecked_mut() };

        if data.len() != Contributor::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
