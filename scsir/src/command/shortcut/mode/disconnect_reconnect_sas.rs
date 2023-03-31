#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const DISCONNECT_RECONNECT_SAS_PAGE_CODE: u8 = 0x02;
pub const DISCONNECT_RECONNECT_SAS_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct DisconnectReconnectSasPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B16,
    pub bus_inactivity_time_limit: B16,
    reserved_1: B16,
    pub maximum_connect_time_limit: B16,
    pub maximum_burst_size: B16,
    restricted: B8,
    reserved_2: B8,
    pub first_burst_size: B16,
}

impl ModePage for DisconnectReconnectSasPage {
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
            size_of::<DisconnectReconnectSasPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(DisconnectReconnectSasPage))
        );
    }
}
