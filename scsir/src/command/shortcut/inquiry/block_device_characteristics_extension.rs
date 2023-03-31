#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct BlockDeviceCharacteristicsExtension {
    pub utilization_type: u8,
    pub utilization_units: u8,
    pub utilization_interval: u8,
    pub utilization_b: u32,
    pub utilization_a: u32,
}

pub fn block_device_characteristics_extension(
    this: &mut InquiryCommand,
) -> crate::Result<BlockDeviceCharacteristicsExtension> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    Ok(BlockDeviceCharacteristicsExtension {
        utilization_type: body.utilization_type(),
        utilization_units: body.utilization_units(),
        utilization_interval: body.utilization_interval(),
        utilization_b: body.utilization_b(),
        utilization_a: body.utilization_a(),
    })
}

const PAGE_CODE: u8 = 0xB5;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved_0: B8,
    utilization_type: B8,
    utilization_units: B8,
    utilization_interval: B8,
    utilization_b: B32,
    utilization_a: B32,
    reserved_1: B128,
    reserved_2: B128,
    reserved_3: B128,
    reserved_4: B128,
    reserved_5: B128,
    reserved_6: B128,
    reserved_7: B128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_LENGTH: usize = 128;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<Page>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(Page))
        );
    }
}
