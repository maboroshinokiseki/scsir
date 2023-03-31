#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const PROTOCOL_SPECIFIC_PORT_FC_PAGE_CODE: u8 = 0x19;
pub const PROTOCOL_SPECIFIC_PORT_FC_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ProtocolSpecificPortFcpage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    reserved_0: B4,
    pub protocol_identifier: B4,
    pub disable_target_fabric_discovery: B1,
    pub prevent_loop_port_bypass: B1,
    pub disable_discovery: B1,
    pub disable_loop_master: B1,
    pub require_hard_address: B1,
    pub allow_login_without_loop_initialization: B1,
    pub disable_target_initiated_port_enable: B1,
    pub disable_target_originated_loop_initialization: B1,
    reserved_1: B24,
    pub sequence_initiative_resource_recovery_timeout_value: B8,
}

impl ModePage for ProtocolSpecificPortFcpage {
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
            size_of::<ProtocolSpecificPortFcpage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(ProtocolSpecificPortFcpage))
        );
    }
}
