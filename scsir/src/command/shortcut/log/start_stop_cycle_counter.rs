use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const START_STOP_CYCLE_COUNTER_PAGE_CODE: u8 = 0x0E;
pub const START_STOP_CYCLE_COUNTER_SUBPAGE_CODE: u8 = 0x00;

pub enum StartStopCycleCounterParameter {
    DateOfManufacture(DateOfManufacture),
    AccountingDate(AccountingDate),
    SpecifiedCycleCountOverDeviceLifetime(SpecifiedCycleCountOverDeviceLifetime),
    AccumulatedStartStopCycles(AccumulatedStartStopCycles),
    SpecifiedLoadUnloadCountOverDeviceLifetime(SpecifiedLoadUnloadCountOverDeviceLifetime),
    AccumulatedLoadUnloadCycles(AccumulatedLoadUnloadCycles),
    Other(GeneralParameter),
}

#[derive(Clone, Debug)]
pub struct DateOfManufacture {
    pub header: ParameterHeader,
    pub year_of_manufacture: String,
    pub week_of_manufacture: String,
}

#[derive(Clone, Debug)]
pub struct AccountingDate {
    pub header: ParameterHeader,
    pub accounting_date_year: String,
    pub accounting_date_week: String,
}

#[derive(Clone, Debug)]
pub struct SpecifiedCycleCountOverDeviceLifetime {
    pub header: ParameterHeader,
    pub specified_cycle_count_over_device_lifetime: u32,
}

#[derive(Clone, Debug)]
pub struct AccumulatedStartStopCycles {
    pub header: ParameterHeader,
    pub accumulated_start_stop_cycles: u32,
}

#[derive(Clone, Debug)]
pub struct SpecifiedLoadUnloadCountOverDeviceLifetime {
    pub header: ParameterHeader,
    pub specified_load_unload_count_over_device_lifetime: u32,
}

#[derive(Clone, Debug)]
pub struct AccumulatedLoadUnloadCycles {
    pub header: ParameterHeader,
    pub accumulated_load_unload_cycles: u32,
}

impl LogParameter for StartStopCycleCounterParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0001 => {
                let (year, bytes) = get_array::<4>(bytes);
                let (week, bytes) = get_array::<2>(bytes);
                let parameter =
                    StartStopCycleCounterParameter::DateOfManufacture(DateOfManufacture {
                        header,
                        year_of_manufacture: String::from_utf8_lossy(&year).to_string(),
                        week_of_manufacture: String::from_utf8_lossy(&week).to_string(),
                    });
                (parameter, bytes)
            }
            0x0002 => {
                let (year, bytes) = get_array::<4>(bytes);
                let (week, bytes) = get_array::<2>(bytes);
                let parameter = StartStopCycleCounterParameter::AccountingDate(AccountingDate {
                    header,
                    accounting_date_year: String::from_utf8_lossy(&year).to_string(),
                    accounting_date_week: String::from_utf8_lossy(&week).to_string(),
                });
                (parameter, bytes)
            }
            0x0003 => {
                let (array, bytes) = get_array(bytes);
                let parameter =
                    StartStopCycleCounterParameter::SpecifiedCycleCountOverDeviceLifetime(
                        SpecifiedCycleCountOverDeviceLifetime {
                            header,
                            specified_cycle_count_over_device_lifetime: u32::from_be_bytes(array),
                        },
                    );
                (parameter, bytes)
            }
            0x0004 => {
                let (array, bytes) = get_array(bytes);
                let parameter = StartStopCycleCounterParameter::AccumulatedStartStopCycles(
                    AccumulatedStartStopCycles {
                        header,
                        accumulated_start_stop_cycles: u32::from_be_bytes(array),
                    },
                );
                (parameter, bytes)
            }
            0x0005 => {
                let (array, bytes) = get_array(bytes);
                let parameter =
                    StartStopCycleCounterParameter::SpecifiedLoadUnloadCountOverDeviceLifetime(
                        SpecifiedLoadUnloadCountOverDeviceLifetime {
                            header,
                            specified_load_unload_count_over_device_lifetime: u32::from_be_bytes(
                                array,
                            ),
                        },
                    );
                (parameter, bytes)
            }
            0x0006 => {
                let (array, bytes) = get_array(bytes);
                let parameter = StartStopCycleCounterParameter::AccumulatedLoadUnloadCycles(
                    AccumulatedLoadUnloadCycles {
                        header,
                        accumulated_load_unload_cycles: u32::from_be_bytes(array),
                    },
                );
                (parameter, bytes)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (StartStopCycleCounterParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            StartStopCycleCounterParameter::DateOfManufacture(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(p.year_of_manufacture.as_bytes());
                bytes.extend_from_slice(p.week_of_manufacture.as_bytes());

                bytes
            }
            StartStopCycleCounterParameter::AccountingDate(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(p.accounting_date_year.as_bytes());
                bytes.extend_from_slice(p.accounting_date_week.as_bytes());

                bytes
            }
            StartStopCycleCounterParameter::SpecifiedCycleCountOverDeviceLifetime(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes
                    .extend_from_slice(&p.specified_cycle_count_over_device_lifetime.to_be_bytes());

                bytes
            }
            StartStopCycleCounterParameter::AccumulatedStartStopCycles(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(&p.accumulated_start_stop_cycles.to_be_bytes());

                bytes
            }
            StartStopCycleCounterParameter::SpecifiedLoadUnloadCountOverDeviceLifetime(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(
                    &p.specified_load_unload_count_over_device_lifetime
                        .to_be_bytes(),
                );

                bytes
            }
            StartStopCycleCounterParameter::AccumulatedLoadUnloadCycles(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(&p.accumulated_load_unload_cycles.to_be_bytes());

                bytes
            }
            StartStopCycleCounterParameter::Other(p) => {
                let mut bytes = p.header.into_bytes().to_vec();
                bytes.extend_from_slice(&p.value);

                bytes
            }
        }
    }
}
