#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const LOGICAL_UNIT_CONTROL_FC_PAGE_CODE: u8 = 0x18;
pub const LOGICAL_UNIT_CONTROL_FC_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LogicalUnitControlFcPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B4,
    pub protocol_identifier: B4,
    reserved_1: B7,
    pub enable_precise_delivery_checking: B1,
    reserved_2: B32,
}

impl ModePage for LogicalUnitControlFcPage {
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

    const PAGE_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<LogicalUnitControlFcPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(LogicalUnitControlFcPage))
        );
    }
}
