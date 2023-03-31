#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct StandardInquiryData {
    pub peripheral_qualifier: u8,
    pub peripheral_device_type: u8,
    pub removable_media: bool,
    pub version: u8,
    pub normal_aca_supported: bool,
    pub hierarchical_support: bool,
    pub response_data_format: u8,
    pub scc_supported: bool,
    pub access_controls_coordinator: bool,
    pub target_port_group_support: u8,
    pub third_party_copy: bool,
    pub protect: bool,
    pub enclosure_services: bool,
    pub multi_port: bool,
    pub command_queuing: bool,
    pub t10_vendor_identification: String,
    pub product_identification: String,
    pub product_revision_level: String,
    pub drive_serial_number: u64,
    pub vendor_unique: [u8; 12],
    pub version_descriptors: [u16; 8],
    pub copyright: String,
}

pub fn standard_inquiry(this: &mut InquiryCommand) -> crate::Result<StandardInquiryData> {
    this.page_code(None);

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(0)?;
    let remaining =
        (result.get_body().additional_length() + 5).saturating_sub(size_of::<PageHeader>() as u8);
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining as usize)?
    };

    let body = result.get_body();

    let vendor_unique = body.vendor_unique().to_be_bytes();
    let vendor_unique: [u8; 12] = vendor_unique[0..12].try_into().unwrap();

    Ok(StandardInquiryData {
        peripheral_qualifier: body.peripheral_qualifier(),
        peripheral_device_type: body.peripheral_device_type(),
        removable_media: body.removable_media() != 0,
        version: body.version(),
        normal_aca_supported: body.normal_aca_supported() != 0,
        hierarchical_support: body.hierarchical_support() != 0,
        response_data_format: body.response_data_format(),
        scc_supported: body.scc_supported() != 0,
        access_controls_coordinator: body.access_controls_coordinator() != 0,
        target_port_group_support: body.target_port_group_support(),
        third_party_copy: body.third_party_copy() != 0,
        protect: body.protect() != 0,
        enclosure_services: body.enclosure_services() != 0,
        multi_port: body.multi_port() != 0,
        command_queuing: body.command_queuing() != 0,
        t10_vendor_identification: String::from_utf8_lossy(
            &body.t10_vendor_identification().to_be_bytes(),
        )
        .to_string(),
        product_identification: String::from_utf8_lossy(
            &body.product_identification().to_be_bytes(),
        )
        .to_string(),
        product_revision_level: String::from_utf8_lossy(
            &body.product_revision_level().to_be_bytes(),
        )
        .to_string(),
        drive_serial_number: body.drive_serial_number(),
        vendor_unique,
        version_descriptors: [
            body.version_descriptor_1(),
            body.version_descriptor_2(),
            body.version_descriptor_3(),
            body.version_descriptor_4(),
            body.version_descriptor_5(),
            body.version_descriptor_6(),
            body.version_descriptor_7(),
            body.version_descriptor_8(),
        ],
        copyright: unsafe { String::from_utf8_lossy(result.elements_as_slice()).to_string() },
    })
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    removable_media: B1,
    reserved_0: B7,
    version: B8,
    obsolete_0: B2,
    normal_aca_supported: B1,
    hierarchical_support: B1,
    response_data_format: B4,
    // Total length - 5
    additional_length: B8,
    scc_supported: B1,
    access_controls_coordinator: B1,
    target_port_group_support: B2,
    third_party_copy: B1,
    reserved_1: B2,
    protect: B1,
    obsolete_1: B1,
    enclosure_services: B1,
    vs_0: B1,
    multi_port: B1,
    obsolete_2: B10,
    command_queuing: B1,
    vs_1: B1,
    t10_vendor_identification: B64,
    product_identification: B128,
    product_revision_level: B32,
    drive_serial_number: B64,
    vendor_unique: B96,
    reserved_2: B16,
    version_descriptor_1: B16,
    version_descriptor_2: B16,
    version_descriptor_3: B16,
    version_descriptor_4: B16,
    version_descriptor_5: B16,
    version_descriptor_6: B16,
    version_descriptor_7: B16,
    version_descriptor_8: B16,
    reserved_3: B128,
    reserved_4: B48,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 96;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );
    }
}
