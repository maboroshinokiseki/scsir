#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const INFORMATIONAL_EXCEPTIONS_CONTROL_PAGE_CODE: u8 = 0x1C;
pub const INFORMATIONAL_EXCEPTIONS_CONTROL_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct InformationalExceptionsControlPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub performance: B1,
    reserved_0: B1,
    pub enable_background_function: B1,
    pub enable_warning: B1,
    pub disable_exception_control: B1,
    pub test: B1,
    pub enable_background_error: B1,
    pub log_error: B1,
    reserved_1: B4,
    pub method_of_reporting_informational_exceptions: B4,
    pub interval_timer: B32,
    pub report_count: B32,
}

impl ModePage for InformationalExceptionsControlPage {
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
            size_of::<InformationalExceptionsControlPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(InformationalExceptionsControlPage))
        );
    }
}
