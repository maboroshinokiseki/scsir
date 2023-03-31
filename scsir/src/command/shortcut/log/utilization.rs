use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const UTILIZATION_PAGE_CODE: u8 = 0x0E;
pub const UTILIZATION_SUBPAGE_CODE: u8 = 0x01;

pub enum UtilizationParameter {
    WorkloadUtilization(WorkloadUtilization),
    UtilizationRateBasedOnDateAndTime(UtilizationRateBasedOnDateAndTime),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct WorkloadUtilization {
    pub header: ParameterHeader,
    pub workload_utilization: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct UtilizationRateBasedOnDateAndTime {
    pub header: ParameterHeader,
    pub date_and_time_based_utilization_rate: B8,
    pub reserved: B8,
}

impl LogParameter for UtilizationParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000 => {
                let (array, left) = get_array(bytes);
                let parameter = UtilizationParameter::WorkloadUtilization(
                    WorkloadUtilization::from_bytes(array),
                );
                (parameter, left)
            }
            0x0001 => {
                let (array, left) = get_array(bytes);
                let parameter = UtilizationParameter::UtilizationRateBasedOnDateAndTime(
                    UtilizationRateBasedOnDateAndTime::from_bytes(array),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (UtilizationParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            UtilizationParameter::WorkloadUtilization(p) => p.bytes.to_vec(),
            UtilizationParameter::UtilizationRateBasedOnDateAndTime(p) => p.bytes.to_vec(),
            UtilizationParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const WORKLOAD_UTILIZATION_LENGTH: usize = 6;
    const UTILIZATION_RATE_BASED_ON_DATE_AND_TIME_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<WorkloadUtilization>(),
            WORKLOAD_UTILIZATION_LENGTH,
            concat!("Size of: ", stringify!(WorkloadUtilization))
        );

        assert_eq!(
            size_of::<UtilizationRateBasedOnDateAndTime>(),
            UTILIZATION_RATE_BASED_ON_DATE_AND_TIME_LENGTH,
            concat!("Size of: ", stringify!(UtilizationRateBasedOnDateAndTime))
        );
    }
}
