#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::LogParameter;

pub const SUPPORTED_LOG_PAGES_AND_SUBPAGES_PAGE_CODE: u8 = 0x00;
pub const SUPPORTED_LOG_PAGES_AND_SUBPAGES_SUBPAGE_CODE: u8 = 0xFF;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SupportedLogPagesAndSubpagesParameter {
    reserved: B2,
    pub page_code: B6,
    pub subpage_code: B8,
}

impl LogParameter for SupportedLogPagesAndSubpagesParameter {
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

    const PARAMETER_LENGTH: usize = 2;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SupportedLogPagesAndSubpagesParameter>(),
            PARAMETER_LENGTH,
            concat!(
                "Size of: ",
                stringify!(SupportedLogPagesAndSubpagesParameter)
            )
        );
    }
}
