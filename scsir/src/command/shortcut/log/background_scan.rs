#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const BACKGROUND_SCAN_PAGE_CODE: u8 = 0x15;
pub const BACKGROUND_SCAN_SUBPAGE_CODE: u8 = 0x00;

#[derive(Clone, Debug)]
pub enum BackgroundScanParameter {
    BackgroundScanStatus(BackgroundScanStatus),
    BackgroundScan(BackgroundScan),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct BackgroundScanStatus {
    pub header: ParameterHeader,
    pub accumulated_power_on_minutes: B32,
    reserved: B8,
    pub background_scan_status: B8,
    pub number_of_background_scans_performed: B16,
    pub background_medium_scan_progress: B16,
    pub number_of_background_medium_scans_performed: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct BackgroundScan {
    pub header: ParameterHeader,
    pub accumulated_power_on_minutes: B32,
    pub reassign_status: B4,
    pub sense_key: B4,
    pub additional_sense_code: B8,
    pub additional_sense_code_qualifier: B8,
    pub vendor_specific: B40,
    pub logical_block_address: B64,
}

impl LogParameter for BackgroundScanParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000 => {
                let (array, left) = get_array(bytes);
                let parameter = BackgroundScanParameter::BackgroundScanStatus(
                    BackgroundScanStatus::from_bytes(array),
                );
                (parameter, left)
            }
            0x0001..=0x0800 => {
                let (array, left) = get_array(bytes);
                let parameter =
                    BackgroundScanParameter::BackgroundScan(BackgroundScan::from_bytes(array));
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (BackgroundScanParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BackgroundScanParameter::BackgroundScanStatus(p) => p.bytes.to_vec(),
            BackgroundScanParameter::BackgroundScan(p) => p.bytes.to_vec(),
            BackgroundScanParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const BACKGROUND_SCAN_STATUS_LENGTH: usize = 16;
    const BACKGROUND_SCAN_LENGTH: usize = 24;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<BackgroundScanStatus>(),
            BACKGROUND_SCAN_STATUS_LENGTH,
            concat!("Size of: ", stringify!(BackgroundScanStatus))
        );

        assert_eq!(
            size_of::<BackgroundScan>(),
            BACKGROUND_SCAN_LENGTH,
            concat!("Size of: ", stringify!(BackgroundScan))
        );
    }
}
