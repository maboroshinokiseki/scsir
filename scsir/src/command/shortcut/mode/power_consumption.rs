#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const POWER_CONSUMPTION_PAGE_CODE: u8 = 0x1A;
pub const POWER_CONSUMPTION_SUBPAGE_CODE: u8 = 0x01;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PowerConsumptionPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B16,
    reserved_1: B6,
    pub active_level: B2,
    pub power_consumption_identifier: B8,
    reserved_2: B64,
}

impl ModePage for PowerConsumptionPage {
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
            size_of::<PowerConsumptionPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(PowerConsumptionPage))
        );
    }
}
