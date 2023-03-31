use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const NON_MEDIUM_ERROR_PAGE_CODE: u8 = 0x06;
pub const NON_MEDIUM_ERROR_SUBPAGE_CODE: u8 = 0x00;

pub enum NonMediumErrorParameter {
    NonMediumError(Box<NonMediumError>),
    Other(GeneralParameter),
}

#[derive(Clone, Copy, Debug)]
pub struct NonMediumError {
    pub header: ParameterHeader,
    pub non_medium_error_count: [u8; 252],
}

impl LogParameter for NonMediumErrorParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, head_left) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let result = match header.parameter_code() {
            0x0000 => {
                let (array, bytes) = get_array(head_left);

                (
                    NonMediumErrorParameter::NonMediumError(Box::new(NonMediumError {
                        header,
                        non_medium_error_count: array,
                    })),
                    bytes,
                )
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (NonMediumErrorParameter::Other(parameter), left)
            }
        };

        result
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            NonMediumErrorParameter::NonMediumError(p) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&p.header.into_bytes());
                bytes.extend_from_slice(&p.non_medium_error_count);

                bytes
            }
            NonMediumErrorParameter::Other(p) => p.to_bytes(),
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
        let dummy = NonMediumError {
            header: ParameterHeader::new(),
            non_medium_error_count: [0; 252],
        };

        assert_eq!(
            size_of_val(&dummy.header) + size_of_val(&dummy.non_medium_error_count),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(NonMediumError))
        );
    }
}
