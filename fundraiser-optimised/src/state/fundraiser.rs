use pinocchio::{
    account_info::AccountInfo,  pubkey::Pubkey, 
};

// --Data--
// maker: Pubkey
// mint_to_raise: pubkey
// amount_to_raise: u64
// amount_raised: u64
// time_ending: i64
// bump: u8

pub struct Fundraiser(*mut u8);

impl Fundraiser {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1;

    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_mut_data_unchecked().as_mut_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(account_info.owner(), &crate::ID);
        Self::from_account_info_unchecked(account_info)
    }

    pub fn maker(&self) -> Pubkey {
        unsafe { *(self.0 as *const Pubkey) }
    }
    pub fn mint_to_raise(&self) -> Pubkey {
        unsafe { *(self.0.add(32) as *const Pubkey) } 
    }
    pub fn amount_to_raise(&self) -> u64 {
        unsafe { *(self.0.add(64) as *const u64) } 
    }
    pub fn amount_raised(&self) -> u64 {
        unsafe { *(self.0.add(72) as *const u64) } 
    }
    pub fn time_ending(&self) -> i64 {
        unsafe { *(self.0.add(80) as *const i64) } 
    }
    pub fn bump(&self) -> u8 {
        unsafe { *(self.0.add(88) as *const u8) } 
    }

}