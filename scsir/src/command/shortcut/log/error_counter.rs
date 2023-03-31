pub const READ_ERROR_COUNTER_PAGE_CODE: u8 = 0x03;
pub const READ_ERROR_COUNTER_SUBPAGE_CODE: u8 = 0x00;
pub const VERIFY_ERROR_COUNTER_PAGE_CODE: u8 = 0x05;
pub const VERIFY_ERROR_COUNTER_SUBPAGE_CODE: u8 = 0x00;
pub const WRITE_ERROR_COUNTER_PAGE_CODE: u8 = 0x02;
pub const WRITE_ERROR_COUNTER_SUBPAGE_CODE: u8 = 0x00;

pub use super::GeneralParameter as ErrorCounterParameter;
