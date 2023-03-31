use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const POWER_CONDITION_TRANSITIONS_PAGE_CODE: u8 = 0x1A;
pub const POWER_CONDITION_TRANSITIONS_SUBPAGE_CODE: u8 = 0x00;

pub enum PowerConditionTransitionsParameter {
    PowerConditionTransitions(PowerConditionTransitions),
    Other(GeneralParameter),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PowerConditionTransitions {
    pub header: ParameterHeader,
    pub parameter_value: B32,
}

impl LogParameter for PowerConditionTransitionsParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, _) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0001..=0x0009 => {
                let (array, left) = get_array(bytes);
                let parameter = PowerConditionTransitionsParameter::PowerConditionTransitions(
                    PowerConditionTransitions::from_bytes(array),
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
            PowerConditionTransitionsParameter::PowerConditionTransitions(p) => p.bytes.to_vec(),
            PowerConditionTransitionsParameter::Other(p) => p.to_bytes(),
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
            size_of::<PowerConditionTransitions>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(PowerConditionTransitions))
        );
    }
}
