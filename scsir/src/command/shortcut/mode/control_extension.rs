#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const CONTROL_EXTENSION_PAGE_CODE: u8 = 0x0A;
pub const CONTROL_EXTENSION_SUBPAGE_CODE: u8 = 0x01;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ControlExtensionPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B5,
    pub timestamp_changeable_by_methods_outside_this_manual: B1,
    pub scsi_precedence: B1,
    pub implicit_asymmetric_logical_unit_access_enabled: B1,
    reserved_1: B4,
    pub initial_command_priority: B4,
    pub maximum_sense_data_length: B8,
    reserved_2: B128,
    reserved_3: B72,
}

impl ModePage for ControlExtensionPage {
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

    const PAGE_LENGTH: usize = 32;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ControlExtensionPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(ControlExtensionPage))
        );
    }
}
