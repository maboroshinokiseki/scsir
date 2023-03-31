#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{LogParameter, ParameterHeader};

pub const SELF_TEST_RESULTS_PAGE_CODE: u8 = 0x10;
pub const SELF_TEST_RESULTS_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SelfTestResultsParameter {
    pub header: ParameterHeader,
    pub self_test_code: B3,
    reserved_0: B1,
    pub self_test_results: B4,
    pub self_test_number: B8,
    pub accumulated_power_on_hours: B16,
    pub address_of_first_failure: B64,
    reserved_1: B4,
    pub sense_key: B4,
    pub additional_sense_code: B8,
    pub additional_sense_code_qualifier: B8,
    pub vendor_specific: B8,
}

impl LogParameter for SelfTestResultsParameter {
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

    const PARAMETER_LENGTH: usize = 20;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SelfTestResultsParameter>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(SelfTestResultsParameter))
        );
    }
}
