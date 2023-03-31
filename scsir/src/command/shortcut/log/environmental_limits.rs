use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const ENVIRONMENTAL_LIMITS_PAGE_CODE: u8 = 0x0D;
pub const ENVIRONMENTAL_LIMITS_SUBPAGE_CODE: u8 = 0x02;

pub enum EnvironmentalLimitsParameter {
    TemperatureLimits(TemperatureLimits),
    RelativeHumidityLimits(RelativeHumidityLimits),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct TemperatureLimits {
    pub header: ParameterHeader,
    pub high_critical_temperature_limit_trigger: B8,
    pub high_critical_temperature_limit_reset: B8,
    pub low_critical_temperature_limit_reset: B8,
    pub low_critical_temperature_limit_trigger: B8,
    pub high_operating_temperature_limit_trigger: B8,
    pub high_operating_temperature_limit_reset: B8,
    pub low_operating_temperature_limit_reset: B8,
    pub low_operating_temperature_limit_trigger: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct RelativeHumidityLimits {
    pub header: ParameterHeader,
    pub high_critical_relative_humidity_limit_trigger: B8,
    pub high_critical_relative_humidity_limit_reset: B8,
    pub low_critical_relative_humidity_limit_reset: B8,
    pub low_critical_relative_humidity_limit_trigger: B8,
    pub high_operating_relative_humidity_limit_trigger: B8,
    pub high_operating_relative_humidity_limit_reset: B8,
    pub low_operating_relative_humidity_limit_reset: B8,
    pub low_operating_relative_humidity_limit_trigger: B8,
}

impl LogParameter for EnvironmentalLimitsParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000..=0x00FF => {
                let (array, left) = get_array(bytes);
                let parameter = EnvironmentalLimitsParameter::TemperatureLimits(
                    TemperatureLimits::from_bytes(array),
                );
                (parameter, left)
            }
            0x0100..=0x01FF => {
                let (array, left) = get_array(bytes);
                let parameter = EnvironmentalLimitsParameter::RelativeHumidityLimits(
                    RelativeHumidityLimits::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (EnvironmentalLimitsParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            EnvironmentalLimitsParameter::TemperatureLimits(p) => p.bytes.to_vec(),
            EnvironmentalLimitsParameter::RelativeHumidityLimits(p) => p.bytes.to_vec(),
            EnvironmentalLimitsParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const TEMPERATURE_LIMITS_LENGTH: usize = 12;
    const RELATIVE_HUMIDITY_LIMITS_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<TemperatureLimits>(),
            TEMPERATURE_LIMITS_LENGTH,
            concat!("Size of: ", stringify!(TemperatureLimits))
        );

        assert_eq!(
            size_of::<RelativeHumidityLimits>(),
            RELATIVE_HUMIDITY_LIMITS_LENGTH,
            concat!("Size of: ", stringify!(RelativeHumidityLimits))
        );
    }
}
