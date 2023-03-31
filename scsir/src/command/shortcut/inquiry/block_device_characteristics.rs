#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct BlockDeviceCharacteristics {
    pub medium_rotation_rate: u16,
    pub product_type: u8,
    pub write_after_block_erase_required: u8,
    pub write_after_cryptographic_erase_required: u8,
    pub nominal_form_factor: u8,
    pub zoned: u8,
    pub background_operation_control_supported: bool,
    pub fuab: bool,
    pub vbuls: bool,
}

pub fn block_device_characteristics(
    this: &mut InquiryCommand,
) -> crate::Result<BlockDeviceCharacteristics> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    Ok(BlockDeviceCharacteristics {
        medium_rotation_rate: body.medium_rotation_rate(),
        product_type: body.product_type(),
        write_after_block_erase_required: body.write_after_block_erase_required(),
        write_after_cryptographic_erase_required: body.write_after_cryptographic_erase_required(),
        nominal_form_factor: body.nominal_form_factor(),
        zoned: body.zoned(),
        background_operation_control_supported: body.background_operation_control_supported() != 0,
        fuab: body.fuab() != 0,
        vbuls: body.vbuls() != 0,
    })
}

const PAGE_CODE: u8 = 0xB1;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    medium_rotation_rate: B16,
    product_type: B8,
    write_after_block_erase_required: B2,
    write_after_cryptographic_erase_required: B2,
    nominal_form_factor: B4,
    reserved_0: B2,
    zoned: B2,
    reserved_1: B1,
    background_operation_control_supported: B1,
    fuab: B1,
    vbuls: B1,
    reserved_2: B128,
    reserved_3: B128,
    reserved_4: B128,
    reserved_5: B56,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_LENGTH: usize = 64;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<Page>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(Page))
        );
    }
}
