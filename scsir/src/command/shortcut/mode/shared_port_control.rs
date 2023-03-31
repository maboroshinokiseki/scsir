#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const SHARED_PORT_CONTROL_PAGE_CODE: u8 = 0x19;
pub const SHARED_PORT_CONTROL_SUBPAGE_CODE: u8 = 0x02;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SharedPortControlPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B8,
    reserved_1: B4,
    pub protocol_identifier: B4,
    pub power_loss_timeout: B16,
    reserved_2: B8,
    pub power_grant_timeout: B8,
    reserved_3: B48,
}

impl ModePage for SharedPortControlPage {
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
            size_of::<SharedPortControlPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(SharedPortControlPage))
        );
    }
}
