#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const IO_ADVICE_HINTS_GROUPING_PAGE_CODE: u8 = 0x0A;
pub const IO_ADVICE_HINTS_GROUPING_SUBPAGE_CODE: u8 = 0x05;

#[derive(Clone, Debug)]
pub struct IoAdviceHintsGroupingPage {
    header: IoAdviceHintsGroupingPageHeader,
    descriptors: Vec<IoAdviceHintsGroupingPageDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct IoAdviceHintsGroupingPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved: B96,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct IoAdviceHintsGroupingPageDescriptor {
    pub io_advice_hints_mode: B2,
    reserved_0: B4,
    pub cache_segment_enable: B1,
    pub information_collection_enable: B1,
    reserved_1: B24,
    pub logical_block_markup_descriptor: B96,
}

impl ModePage for IoAdviceHintsGroupingPage {
    fn new() -> Self {
        Self {
            header: IoAdviceHintsGroupingPageHeader::new(),
            descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);

        let header = IoAdviceHintsGroupingPageHeader::from_bytes(array);

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            if descriptors.len() == DESCRIPTOR_COUNT {
                break;
            }

            let array;
            (array, bytes) = get_array(bytes);
            let descriptor = IoAdviceHintsGroupingPageDescriptor::from_bytes(array);
            descriptors.push(descriptor);
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

const DESCRIPTOR_COUNT: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 16;
    const PAGE_DESCRIPTOR_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<IoAdviceHintsGroupingPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(IoAdviceHintsGroupingPageHeader))
        );

        assert_eq!(
            size_of::<IoAdviceHintsGroupingPageDescriptor>(),
            PAGE_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(IoAdviceHintsGroupingPageDescriptor))
        );
    }
}
