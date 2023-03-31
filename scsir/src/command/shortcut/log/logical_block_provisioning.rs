#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const LOGICAL_BLOCK_PROVISIONING_PAGE_CODE: u8 = 0x0C;
pub const LOGICAL_BLOCK_PROVISIONING_SUBPAGE_CODE: u8 = 0x00;

pub enum LogicalBlockProvisioningParameter {
    LogicalBlockProvisioning(LogicalBlockProvisioning),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LogicalBlockProvisioning {
    pub header: ParameterHeader,
    pub resource_count: B32,
    reserved_0: B7,
    pub scope: B1,
    reserved_1: B24,
}

impl LogParameter for LogicalBlockProvisioningParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0001 | 0x0002 | 0x0003 | 0x0100 | 0x0101 | 0x0102 => {
                let (array, left) = get_array(bytes);
                let parameter = LogicalBlockProvisioningParameter::LogicalBlockProvisioning(
                    LogicalBlockProvisioning::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (LogicalBlockProvisioningParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            LogicalBlockProvisioningParameter::LogicalBlockProvisioning(p) => p.bytes.to_vec(),
            LogicalBlockProvisioningParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PARAMETER_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<LogicalBlockProvisioning>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(LogicalBlockProvisioning))
        );
    }
}
