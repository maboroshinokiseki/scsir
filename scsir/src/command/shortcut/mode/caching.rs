#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const CACHING_PAGE_CODE: u8 = 0x08;
pub const CACHING_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct CachingPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub initiator_control: B1,
    pub abort_prefetch: B1,
    pub caching_analysis_permitted: B1,
    pub discontinuity: B1,
    pub size_enable: B1,
    pub write_cache_enable: B1,
    pub multiplication_factor: B1,
    pub read_cache_disable: B1,
    pub demand_read_retention_priority: B4,
    pub write_retention_priority: B4,
    pub disable_prefetch_transfer_length: B16,
    pub minimum_prefetch: B16,
    pub maximum_prefetch: B16,
    pub maximum_prefetch_ceiling: B16,
    pub force_sequential_write: B1,
    pub logical_block_cache_segment_size: B1,
    pub disable_read_ahead: B1,
    pub vendor_specific: B2,
    pub sync_prog: B2,
    pub non_volatile_cache_disabled: B1,
    pub number_of_cache_segments: B8,
    pub cache_segment_size: B16,
    reserved: B8,
    obsolete: B24,
}

impl ModePage for CachingPage {
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

    const PAGE_LENGTH: usize = 20;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CachingPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(CachingPage))
        );
    }
}
