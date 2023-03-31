use crate::command::get_array;

use super::{GeneralParameter, LogParameter, ParameterHeader};

pub const BACKGROUND_OPERATION_PAGE_CODE: u8 = 0x15;
pub const BACKGROUND_OPERATION_SUBPAGE_CODE: u8 = 0x02;

#[derive(Clone, Debug)]
pub enum BackgroundOperationParameter {
    BackgroundOperation(BackgroundOperation),
    Other(GeneralParameter),
}

#[derive(Clone, Debug)]
pub struct BackgroundOperation {
    pub header: ParameterHeader,
    pub background_operation_status: u8,
    pub extra: Vec<u8>,
}

impl LogParameter for BackgroundOperationParameter {
    fn new() -> Self {
        Self::Other(GeneralParameter::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, left) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);

        match header.parameter_code() {
            0x0000 => {
                let (array, left) = get_array::<1>(left);
                let background_operation_status = array[0];
                let (extra, left) =
                    left.split_at(usize::min(left.len(), header.parameter_length() as usize));
                let parameter =
                    BackgroundOperationParameter::BackgroundOperation(BackgroundOperation {
                        header,
                        background_operation_status,
                        extra: extra.to_vec(),
                    });

                (parameter, left)
            }
            _ => {
                let (parameter, left) = GeneralParameter::from_bytes(bytes);
                (BackgroundOperationParameter::Other(parameter), left)
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            BackgroundOperationParameter::BackgroundOperation(p) => {
                let mut bytes = vec![];
                bytes.extend_from_slice(&p.header.into_bytes());
                bytes.push(p.background_operation_status);
                bytes.extend_from_slice(&p.extra);

                bytes
            }
            BackgroundOperationParameter::Other(p) => p.to_bytes(),
        }
    }
}
