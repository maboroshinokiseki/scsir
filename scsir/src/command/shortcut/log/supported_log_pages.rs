use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::LogParameter;

pub const SUPPORTED_LOG_PAGES_PAGE_CODE: u8 = 0x00;
pub const SUPPORTED_LOG_PAGES_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SupportedLogPagesParameter {
    pub page_code: B8,
}

impl LogParameter for SupportedLogPagesParameter {
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

    const PARAMETER_LENGTH: usize = 1;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SupportedLogPagesParameter>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(SupportedLogPagesParameter))
        );
    }
}
