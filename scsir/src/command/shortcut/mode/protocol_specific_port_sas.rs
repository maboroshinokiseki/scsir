#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const PROTOCOL_SPECIFIC_PORT_SAS_PAGE_CODE: u8 = 0x19;
pub const PROTOCOL_SPECIFIC_PORT_SAS_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ProtocolSpecificPortSasPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B1,
    pub continue_awt: B1,
    pub broadcast_asynchronous_event: B1,
    pub ready_led_meaning: B1,
    pub protocol_identifier: B4,
    reserved_1: B8,
    pub i_t_nexus_loss_time: B16,
    pub initiator_response_timeout: B16,
    pub reject_to_open_limit: B16,
    pub maximum_allowed_xfer_rdy: B8,
    reserved_2: B40,
}

impl ModePage for ProtocolSpecificPortSasPage {
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
            size_of::<ProtocolSpecificPortSasPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(ProtocolSpecificPortSasPage))
        );
    }
}
