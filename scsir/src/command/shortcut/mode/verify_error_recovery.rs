#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const VERIFY_ERROR_RECOVERY_PAGE_CODE: u8 = 0x07;
pub const VERIFY_ERROR_RECOVERY_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct VerifyErrorRecoveryPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B4,
    pub enable_early_recovery: B1,
    pub post_error: B1,
    pub data_terminate_on_error: B1,
    pub disable_correction: B1,
    pub verify_retry_count: B8,
    obsolete: B8,
    reserved_1: B40,
    pub verify_recovery_time_limit: B16,
}

impl ModePage for VerifyErrorRecoveryPage {
    fn new() -> Self {
        Self::new()
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);

        (Self::from_bytes(array), bytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<VerifyErrorRecoveryPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(VerifyErrorRecoveryPage))
        );
    }
}
