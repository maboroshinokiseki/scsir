#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const DISCONNECT_RECONNECT_FC_PAGE_CODE: u8 = 0x02;
pub const DISCONNECT_RECONNECT_FC_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct DisconnectReconnectFcPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub buffer_full_ratio: B8,
    pub buffer_empty_ratio: B8,
    pub bus_inactivity_limit: B16,
    pub disconnect_time_limit: B16,
    pub connect_time_limit: B16,
    pub maximum_burst_size: B16,
    pub enable_modify_data_pointers: B1,
    pub faa: B1,
    pub fab: B1,
    pub fac: B1,
    restricted: B4,
    reserved: B8,
    pub first_burst_size: B16,
}

impl ModePage for DisconnectReconnectFcPage {
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
            size_of::<DisconnectReconnectFcPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(DisconnectReconnectFcPage))
        );
    }
}
