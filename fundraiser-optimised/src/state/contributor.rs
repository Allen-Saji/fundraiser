use pinocchio::account_info::AccountInfo;

// --data--
// amount: u64

pub struct Contributor(*const u8);

impl Contributor {
    pub const LEN: usize = 8;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(account_info.owner(), &crate::ID);
        Self::from_account_info_unchecked(account_info)
    }

    pub fn amount(&self) -> u64 {
        unsafe { *(self.0 as *const u64) }
    }
}