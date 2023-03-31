#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{page_header, ModePage};

pub const ENHANCED_PHY_CONTROL_PAGE_CODE: u8 = 0x19;
pub const ENHANCED_PHY_CONTROL_SUBPAGE_CODE: u8 = 0x03;

#[derive(Clone, Debug)]
pub struct EnhancedPhyControlPage {
    header: EnhancedPhyControlPageHeader,
    descriptors: Vec<EnhancedPhyControlPageDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct EnhancedPhyControlPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B8,
    reserved_1: B4,
    pub protocol_identifier: B4,
    pub generation_code: B8,
    pub number_of_phys: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct EnhancedPhyControlPageDescriptor {
    reserved_0: B8,
    pub phy_identifier: B8,
    pub descriptor_length: B16,
    pub programmed_phy_capabilities: B32,
    pub current_phy_capabilities: B32,
    pub attached_phy_capabilities: B32,
    reserved_1: B16,
    reserved_2: B3,
    pub negotiated_ssc: B1,
    pub negotiated_physical_link_rate: B4,
    reserved_3: B7,
    pub hardware_muxing_supported: B1,
}

impl ModePage for EnhancedPhyControlPage {
    fn new() -> Self {
        Self {
            header: EnhancedPhyControlPageHeader::new(),
            descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);

        let header = EnhancedPhyControlPageHeader::from_bytes(array);
        let descriptor_count = (header.page_length() as usize
            + size_of::<page_header::CommomSubpageHeader>()
            - size_of::<EnhancedPhyControlPageHeader>())
            / size_of::<EnhancedPhyControlPageDescriptor>();

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            if descriptors.len() == descriptor_count {
                break;
            }

            let array;
            (array, bytes) = get_array(bytes);
            let descriptor = EnhancedPhyControlPageDescriptor::from_bytes(array);
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

    const PAGE_HEADER_LENGTH: usize = 8;
    const PAGE_DESCRIPTOR_LENGTH: usize = 20;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<EnhancedPhyControlPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(EnhancedPhyControlPageHeader))
        );

        assert_eq!(
            size_of::<EnhancedPhyControlPageDescriptor>(),
            PAGE_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(EnhancedPhyControlPageDescriptor))
        );
    }
}
