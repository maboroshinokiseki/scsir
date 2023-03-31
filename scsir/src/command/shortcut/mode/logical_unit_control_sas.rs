#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const LOGICAL_UNIT_CONTROL_SAS_PAGE_CODE: u8 = 0x18;
pub const LOGICAL_UNIT_CONTROL_SAS_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LogicalUnitControlSasPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B3,
    pub transport_layer_retries: B1,
    pub protocol_identifier: B4,
    reserved_1: B40,
}

impl ModePage for LogicalUnitControlSasPage {
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
            size_of::<LogicalUnitControlSasPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(LogicalUnitControlSasPage))
        );
    }
}
