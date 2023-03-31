use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const APPLICATION_CLIENT_PAGE_CODE: u8 = 0x0F;
pub const APPLICATION_CLIENT_SUBPAGE_CODE: u8 = 0x00;

#[derive(Clone, Debug)]
pub enum ApplicationClientParameter {
    GeneralUsageApplicationClient(Box<GeneralUsageApplicationClient>),
    Other(GeneralParameter),
}

#[derive(Clone, Copy, Debug)]
pub struct GeneralUsageApplicationClient {
    pub header: ParameterHeader,
    pub general_usage_parameter_bytes: [u8; 252],
}

impl LogParameter for ApplicationClientParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, left) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);

        match header.parameter_code() {
            0x0000..=0x0FFF => {
                let (array, left) = get_array(left);
                let parameter = ApplicationClientParameter::GeneralUsageApplicationClient(
                    Box::new(GeneralUsageApplicationClient {
                        header,
                        general_usage_parameter_bytes: array,
                    }),
                );
                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (ApplicationClientParameter::Other(parameter), left)
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ApplicationClientParameter::GeneralUsageApplicationClient(p) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&p.header.into_bytes());
                bytes.extend_from_slice(&p.general_usage_parameter_bytes);

                bytes
            }
            ApplicationClientParameter::Other(p) => p.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of_val;

    const PARAMETER_LENGTH: usize = 256;

    #[test]
    fn layout_test() {
        let dummy = GeneralUsageApplicationClient {
            header: ParameterHeader::new(),
            general_usage_parameter_bytes: [0; 252],
        };

        assert_eq!(
            size_of_val(&dummy.header) + size_of_val(&dummy.general_usage_parameter_bytes),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(GeneralUsageApplicationClient))
        );
    }
}
