use crate::command::get_array;

use super::{LogParameter, ParameterHeader};

#[derive(Clone, Debug)]
pub struct GeneralParameter {
    pub header: ParameterHeader,
    pub value: Vec<u8>,
}

impl LogParameter for GeneralParameter {
    fn new() -> Self {
        Self {
            header: ParameterHeader::new(),
            value: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let (value, bytes) =
            bytes.split_at(usize::min(bytes.len(), header.parameter_length() as usize));
        (
            Self {
                header,
                value: value.to_vec(),
            },
            bytes,
        )
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.into_bytes());
        bytes.extend_from_slice(&self.value);

        bytes
    }
}
