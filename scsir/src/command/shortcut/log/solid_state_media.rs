#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const SOLID_STATE_MEDIA_PAGE_CODE: u8 = 0x11;
pub const SOLID_STATE_MEDIA_SUBPAGE_CODE: u8 = 0x00;

pub enum SolidStateMediaParameter {
    SolidStateMedia(SolidStateMedia),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SolidStateMedia {
    pub header: ParameterHeader,
    reserved: B24,
    pub percentage_used_endurance_indicator: B8,
}

impl LogParameter for SolidStateMediaParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0001 => {
                let (array, left) = get_array(bytes);
                let parameter =
                    SolidStateMediaParameter::SolidStateMedia(SolidStateMedia::from_bytes(array));
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (SolidStateMediaParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            SolidStateMediaParameter::SolidStateMedia(p) => p.bytes.to_vec(),
            SolidStateMediaParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PARAMETER_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<SolidStateMedia>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(SolidStateMedia))
        );
    }
}
