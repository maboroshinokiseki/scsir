#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const BACKGROUND_OPERATION_CONTROL_PAGE_CODE: u8 = 0x0A;
pub const BACKGROUND_OPERATION_CONTROL_SUBPAGE_CODE: u8 = 0x06;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct BackgroundOperationControlPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    pub background_operation_mode: B2,
    reserved: B6,
}

impl ModePage for BackgroundOperationControlPage {
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

    const PAGE_LENGTH: usize = 5;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<BackgroundOperationControlPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(BackgroundOperationControlPage))
        );
    }
}
