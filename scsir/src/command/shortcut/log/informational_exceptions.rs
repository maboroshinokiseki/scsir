use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const INFORMATIONAL_EXCEPTIONS_PAGE_CODE: u8 = 0x2F;
pub const INFORMATIONAL_EXCEPTIONS_SUBPAGE_CODE: u8 = 0x00;

pub enum InformationalExceptionsParameter {
    InformationalExceptionsGeneral(InformationalExceptionsGeneral),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct InformationalExceptionsGeneral {
    pub header: ParameterHeader,
    pub informational_exception_additional_sense_code: B8,
    pub informational_exception_additional_sense_code_qualifier: B8,
    pub most_recent_temperature_reading: B8,
    pub vendor_hda_temperature_trip_point: B8,
    pub maximum_temperature: B8,
    pub vendor_specific: B24,
}

impl LogParameter for InformationalExceptionsParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000 => {
                let (array, left) = get_array(bytes);
                let parameter = InformationalExceptionsParameter::InformationalExceptionsGeneral(
                    InformationalExceptionsGeneral::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (InformationalExceptionsParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            InformationalExceptionsParameter::InformationalExceptionsGeneral(p) => p.bytes.to_vec(),
            InformationalExceptionsParameter::Other(p) => p.to_bytes(),
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
            size_of::<InformationalExceptionsGeneral>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(InformationalExceptionsGeneral))
        );
    }
}
