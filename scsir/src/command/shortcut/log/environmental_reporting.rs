#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const ENVIRONMENTAL_REPORTING_PAGE_CODE: u8 = 0x0D;
pub const ENVIRONMENTAL_REPORTING_SUBPAGE_CODE: u8 = 0x01;

pub enum EnvironmentalReportingParameter {
    TemperatureReport(TemperatureReport),
    RelativeHumidityReport(RelativeHumidityReport),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct TemperatureReport {
    pub header: ParameterHeader,
    reserved_0: B8,
    pub temperature: B8,
    pub lifetime_maximum_temperature: B8,
    pub lifetime_minimum_temperature: B8,
    pub maximum_temperature_since_power_on: B8,
    pub minimum_temperature_since_power_on: B8,
    reserved_1: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct RelativeHumidityReport {
    pub header: ParameterHeader,
    reserved_0: B8,
    pub relative_humidity: B8,
    pub lifetime_maximum_relative_humidity: B8,
    pub lifetime_minimum_relative_humidity: B8,
    pub maximum_relative_humidity_since_power_on: B8,
    pub minimum_relative_humidity_since_power_on: B8,
    reserved_1: B16,
}

impl LogParameter for EnvironmentalReportingParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000..=0x00FF => {
                let (array, left) = get_array(bytes);
                let parameter = EnvironmentalReportingParameter::TemperatureReport(
                    TemperatureReport::from_bytes(array),
                );
                (parameter, left)
            }
            0x0100..=0x01FF => {
                let (array, left) = get_array(bytes);
                let parameter = EnvironmentalReportingParameter::RelativeHumidityReport(
                    RelativeHumidityReport::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (EnvironmentalReportingParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            EnvironmentalReportingParameter::TemperatureReport(p) => p.bytes.to_vec(),
            EnvironmentalReportingParameter::RelativeHumidityReport(p) => p.bytes.to_vec(),
            EnvironmentalReportingParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const TEMPERATURE_REPORT_LENGTH: usize = 12;
    const RELATIVE_HUMIDITY_REPORT_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<TemperatureReport>(),
            TEMPERATURE_REPORT_LENGTH,
            concat!("Size of: ", stringify!(TemperatureReport))
        );

        assert_eq!(
            size_of::<RelativeHumidityReport>(),
            RELATIVE_HUMIDITY_REPORT_LENGTH,
            concat!("Size of: ", stringify!(RelativeHumidityReport))
        );
    }
}
