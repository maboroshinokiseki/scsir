#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const BACKGROUND_CONTROL_PAGE_CODE: u8 = 0x1C;
pub const BACKGROUND_CONTROL_SUBPAGE_CODE: u8 = 0x01;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct BackgroundControlPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B5,
    pub suspend_on_log_full: B1,
    pub log_only_when_intervention_required: B1,
    pub enable_background_medium_scan: B1,
    reserved_1: B7,
    pub enable_pre_scan: B1,
    pub background_medium_scan_interval_time: B16,
    pub background_pre_scan_time_limit: B16,
    pub minimum_idle_time_before_background_scan: B16,
    pub maximum_time_to_suspend_background_scan: B16,
    reserved_2: B16,
}

impl ModePage for BackgroundControlPage {
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

    const PAGE_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<BackgroundControlPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(BackgroundControlPage))
        );
    }
}
