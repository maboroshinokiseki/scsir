#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const READ_WRITE_ERROR_RECOVERY_PAGE_CODE: u8 = 0x01;
pub const READ_WRITE_ERROR_RECOVERY_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ReadWriteErrorRecoveryPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub automatic_write_reallocation_enabled: B1,
    pub automatic_read_reallocation_enabled: B1,
    pub transfer_block: B1,
    pub read_continuous: B1,
    pub enable_early_recovery: B1,
    pub post_error: B1,
    pub data_terminate_on_error: B1,
    pub disable_correction: B1,
    pub read_retry_count: B8,
    obsolete: B24,
    reserved_0: B8,
    pub write_retry_count: B8,
    reserved_1: B8,
    pub recovery_time_limit: B16,
}

impl ModePage for ReadWriteErrorRecoveryPage {
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
            size_of::<ReadWriteErrorRecoveryPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(ReadWriteErrorRecoveryPage))
        );
    }
}
