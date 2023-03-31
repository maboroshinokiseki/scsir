#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{page_header, ModePage};

pub const LOGICAL_BLOCK_PROVISIONING_PAGE_CODE: u8 = 0x1C;
pub const LOGICAL_BLOCK_PROVISIONING_SUBPAGE_CODE: u8 = 0x02;

#[derive(Clone, Debug)]
pub struct LogicalBlockProvisioningPage {
    header: LogicalBlockProvisioningPageHeader,
    descriptors: Vec<LogicalBlockProvisioningPageDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LogicalBlockProvisioningPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B7,
    pub situa: B1,
    reserved_1: B88,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LogicalBlockProvisioningPageDescriptor {
    pub enabled: B1,
    reserved_0: B1,
    pub threshold_type: B3,
    pub threshold_arming: B3,
    pub threshold_resource: B8,
    reserved_1: B16,
    pub threshold_count: B32,
}

impl ModePage for LogicalBlockProvisioningPage {
    fn new() -> Self {
        Self {
            header: LogicalBlockProvisioningPageHeader::new(),
            descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);

        let header = LogicalBlockProvisioningPageHeader::from_bytes(array);
        let descriptor_count = (header.page_length() as usize
            + size_of::<page_header::CommomSubpageHeader>()
            - size_of::<LogicalBlockProvisioningPageHeader>())
            / size_of::<LogicalBlockProvisioningPageDescriptor>();

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            if descriptors.len() == descriptor_count {
                break;
            }

            let array;
            (array, bytes) = get_array(bytes);
            let descriptor = LogicalBlockProvisioningPageDescriptor::from_bytes(array);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 16;
    const PAGE_DESCRIPTOR_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<LogicalBlockProvisioningPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ApplicationTagPageHeader))
        );

        assert_eq!(
            size_of::<LogicalBlockProvisioningPageDescriptor>(),
            PAGE_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(ApplicationTagPageDescriptor))
        );
    }
}
