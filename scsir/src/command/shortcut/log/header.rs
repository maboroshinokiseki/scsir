#![allow(unused_braces)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PageHeader {
    pub disable_save: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug, BitfieldSpecifier)]
pub struct ParameterHeader {
    pub parameter_code: B16,
    pub parameter_control_byte: B8,
    pub parameter_length: B8,
}

impl PageHeader {
    pub fn from_slice(bytes: &[u8]) -> Self {
        let (array, _) = get_array(bytes);

        PageHeader::from_bytes(array)
    }
}

impl ParameterHeader {
    pub fn from_slice(bytes: &[u8]) -> Self {
        let (array, _) = get_array(bytes);

        ParameterHeader::from_bytes(array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;
    const PARAMETER_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );

        assert_eq!(
            size_of::<ParameterHeader>(),
            PARAMETER_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ParameterHeader))
        );
    }
}
