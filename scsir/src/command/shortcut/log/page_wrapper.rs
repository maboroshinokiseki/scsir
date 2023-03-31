use crate::command::get_array;

use super::header::PageHeader;

#[derive(Clone, Debug)]
pub struct PageWrapper<Parameter: LogParameter> {
    pub header: PageHeader,
    pub parameters: Vec<Parameter>,
}

pub trait LogParameter {
    fn new() -> Self
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8])
    where
        Self: Sized;

    fn to_bytes(&self) -> Vec<u8>;
}

impl<Parameter: LogParameter> PageWrapper<Parameter> {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (arryay, bytes) = get_array(bytes);
        let header = PageHeader::from_bytes(arryay);

        let mut bytes = &bytes[..usize::min(header.page_length() as usize, bytes.len())];
        let mut parameter;
        let mut parameters = vec![];
        while !bytes.is_empty() {
            (parameter, bytes) = Parameter::from_bytes(bytes);
            parameters.push(parameter);
        }

        Self { header, parameters }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.into_bytes());
        for item in &self.parameters {
            bytes.extend_from_slice(&item.to_bytes());
        }

        bytes
    }
}
