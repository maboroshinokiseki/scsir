#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const APPLICATION_TAG_PAGE_CODE: u8 = 0x0A;
pub const APPLICATION_TAG_SUBPAGE_CODE: u8 = 0x02;

#[derive(Clone, Debug)]
pub struct ApplicationTagPage {
    header: ApplicationTagPageHeader,
    descriptors: Vec<ApplicationTagPageDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ApplicationTagPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved: B96,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ApplicationTagPageDescriptor {
    pub last: B1,
    reserved_0: B7,
    reserved_1: B40,
    pub logical_block_application_tag: B16,
    pub logical_block_address: B64,
    pub logical_block_count: B64,
}

impl ModePage for ApplicationTagPage {
    fn new() -> Self {
        Self {
            header: ApplicationTagPageHeader::new(),
            descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);

        let header = ApplicationTagPageHeader::from_bytes(array);

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            let array;
            (array, bytes) = get_array(bytes);
            let descriptor = ApplicationTagPageDescriptor::from_bytes(array);
            descriptors.push(descriptor);

            if descriptor.last() != 0 {
                break;
            }
        }

        (
            Self {
                header,
                descriptors,
            },
            bytes,
        )
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.bytes);

        for item in &self.descriptors {
            bytes.extend_from_slice(&item.bytes);
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 16;
    const PAGE_DESCRIPTOR_LENGTH: usize = 24;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ApplicationTagPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ApplicationTagPageHeader))
        );

        assert_eq!(
            size_of::<ApplicationTagPageDescriptor>(),
            PAGE_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(ApplicationTagPageDescriptor))
        );
    }
}
