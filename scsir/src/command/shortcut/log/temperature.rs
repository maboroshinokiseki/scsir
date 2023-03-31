use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const TEMPERATURE_PAGE_CODE: u8 = 0x0D;
pub const TEMPERATURE_SUBPAGE_CODE: u8 = 0x00;

pub enum PowerConditionTransitionsParameter {
    Temperature(Temperature),
    ReferenceTemperature(ReferenceTemperature),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct Temperature {
    pub header: ParameterHeader,
    pub reserved: B8,
    pub temperature: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ReferenceTemperature {
    pub header: ParameterHeader,
    pub reserved: B8,
    pub reference_temperature: B8,
}

impl LogParameter for PowerConditionTransitionsParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000 => {
                let (array, left) = get_array(bytes);
                let parameter =
                    PowerConditionTransitionsParameter::Temperature(Temperature::from_bytes(array));
                (parameter, left)
            }
            0x0001 => {
                let (array, left) = get_array(bytes);
                let parameter = PowerConditionTransitionsParameter::ReferenceTemperature(
                    ReferenceTemperature::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (PowerConditionTransitionsParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            PowerConditionTransitionsParameter::Temperature(p) => p.bytes.to_vec(),
            PowerConditionTransitionsParameter::ReferenceTemperature(p) => p.bytes.to_vec(),
            PowerConditionTransitionsParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const TEMPERATURE_LENGTH: usize = 6;
    const REFERENCE_TEMPERATURE_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<Temperature>(),
            TEMPERATURE_LENGTH,
            concat!("Size of: ", stringify!(Temperature))
        );

        assert_eq!(
            size_of::<ReferenceTemperature>(),
            REFERENCE_TEMPERATURE_LENGTH,
            concat!("Size of: ", stringify!(ReferenceTemperature))
        );
    }
}
