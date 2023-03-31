#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct LogicalBlockProvisioning {
    pub threshold_exponent: u8,
    pub logical_block_provisioning_unmap: bool,
    pub logical_block_provisioning_write_same: bool,
    pub logical_block_provisioning_write_same_10: bool,
    pub logical_block_provisioning_read_zeros: u8,
    pub anchor_supported: bool,
    pub minimum_percentage: u8,
    pub provisioning_type: u8,
    pub threshold_percentage: u8,
    pub descriptors: Vec<u8>,
}

pub fn logical_block_provisioning(
    this: &mut InquiryCommand,
) -> crate::Result<LogicalBlockProvisioning> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(64 - size_of::<PageHeader>())?;

    let body = result.get_body();

    let descriptor_present = body.descriptor_present() != 0;

    let descriptors = if descriptor_present {
        unsafe { Vec::from(&result.elements_as_slice()[..=body.page_length() as usize - 4]) }
    } else {
        vec![]
    };

    Ok(LogicalBlockProvisioning {
        threshold_exponent: body.threshold_exponent(),
        logical_block_provisioning_unmap: body.logical_block_provisioning_unmap() != 0,
        logical_block_provisioning_write_same: body.logical_block_provisioning_write_same() != 0,
        logical_block_provisioning_write_same_10: body.logical_block_provisioning_write_same_10()
            != 0,
        logical_block_provisioning_read_zeros: body.logical_block_provisioning_read_zeros(),
        anchor_supported: body.anchor_supported() != 0,
        minimum_percentage: body.minimum_percentage(),
        provisioning_type: body.provisioning_type(),
        threshold_percentage: body.threshold_percentage(),
        descriptors,
    })
}

const PAGE_CODE: u8 = 0xB2;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    threshold_exponent: B8,
    logical_block_provisioning_unmap: B1,
    logical_block_provisioning_write_same: B1,
    logical_block_provisioning_write_same_10: B1,
    logical_block_provisioning_read_zeros: B3,
    anchor_supported: B1,
    descriptor_present: B1,
    minimum_percentage: B5,
    provisioning_type: B3,
    threshold_percentage: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );
    }
}
