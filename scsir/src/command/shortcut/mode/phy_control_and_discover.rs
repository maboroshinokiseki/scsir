#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{page_header, ModePage};

pub const PHY_CONTROL_AND_DISCOVER_PAGE_CODE: u8 = 0x19;
pub const PHY_CONTROL_AND_DISCOVER_SUBPAGE_CODE: u8 = 0x01;

#[derive(Clone, Debug)]
pub struct PhyControlAndDiscoverPage {
    header: PhyControlAndDiscoverPageHeader,
    descriptors: Vec<PhyControlAndDiscoverPageDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PhyControlAndDiscoverPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved_0: B8,
    reserved_1: B4,
    protocol_identifier: B4,
    generation_code: B8,
    number_of_phys: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PhyControlAndDiscoverPageDescriptor {
    reserved_00: B8,
    pub phy_identifier: B8,
    reserved_01: B16,
    reserved_02: B1,
    pub attached_device_type: B3,
    reserved_03: B4,
    pub reason: B4,
    pub negotiated_physical_link_rate: B4,
    reserved_04: B4,
    pub attached_ssp_initiator_port: B1,
    pub attached_stp_initiator_port: B1,
    pub attached_smp_initiator_port: B1,
    reserved_05: B1,
    reserved_06: B4,
    pub attached_ssp_target_port: B1,
    pub attached_stp_target_port: B1,
    pub attached_smp_target_port: B1,
    reserved_07: B1,
    pub sas_address: B64,
    pub attached_sas_address: B64,
    pub attached_phy_identifier: B8,
    reserved_08: B56,
    pub programmed_minimum_physical_link_rate: B4,
    pub hardware_minimum_physical_link_rate: B4,
    pub programmed_maximum_physical_link_rate: B4,
    pub hardware_maximum_physical_link_rate: B4,
    reserved_09: B64,
    pub vendor_specific: B16,
    reserved_10: B32,
}

impl ModePage for PhyControlAndDiscoverPage {
    fn new() -> Self {
        Self {
            header: PhyControlAndDiscoverPageHeader::new(),
            descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);

        let header = PhyControlAndDiscoverPageHeader::from_bytes(array);
        let descriptor_count = (header.page_length() as usize
            + size_of::<page_header::CommomSubpageHeader>()
            - size_of::<PhyControlAndDiscoverPageHeader>())
            / size_of::<PhyControlAndDiscoverPageDescriptor>();

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            if descriptors.len() == descriptor_count {
                break;
            }

            let array;
            (array, bytes) = get_array(bytes);
            let descriptor = PhyControlAndDiscoverPageDescriptor::from_bytes(array);
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
    const PAGE_DESCRIPTOR_LENGTH: usize = 48;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PhyControlAndDiscoverPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PhyControlAndDiscoverPageHeader))
        );

        assert_eq!(
            size_of::<PhyControlAndDiscoverPageDescriptor>(),
            PAGE_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(PhyControlAndDiscoverPageDescriptor))
        );
    }
}
