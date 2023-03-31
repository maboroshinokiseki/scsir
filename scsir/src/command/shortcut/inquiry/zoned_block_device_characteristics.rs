#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct ZonedBlockDeviceCharacteristics {
    pub unrestricted_read_in_sequential_write_required_zone: bool,
    pub optimal_number_of_open_sequential_write_preferred_zones: u32,
    pub optimal_number_of_non_sequentially_written_sequential_write_preferred_zones: u32,
    pub maximum_number_of_open_sequential_write_required_zones: u32,
}

pub fn zoned_block_device_characteristics(
    this: &mut InquiryCommand,
) -> crate::Result<ZonedBlockDeviceCharacteristics> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    Ok(ZonedBlockDeviceCharacteristics {
        unrestricted_read_in_sequential_write_required_zone: body
            .unrestricted_read_in_sequential_write_required_zone()
            != 0,
        optimal_number_of_open_sequential_write_preferred_zones: body
            .optimal_number_of_open_sequential_write_preferred_zones(),
        optimal_number_of_non_sequentially_written_sequential_write_preferred_zones: body
            .optimal_number_of_non_sequentially_written_sequential_write_preferred_zones(),
        maximum_number_of_open_sequential_write_required_zones: body
            .maximum_number_of_open_sequential_write_required_zones(),
    })
}

const PAGE_CODE: u8 = 0xB6;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved_0: B7,
    unrestricted_read_in_sequential_write_required_zone: B1,
    reserved_1: B24,
    optimal_number_of_open_sequential_write_preferred_zones: B32,
    optimal_number_of_non_sequentially_written_sequential_write_preferred_zones: B32,
    maximum_number_of_open_sequential_write_required_zones: B32,
    reserved_2: B128,
    reserved_3: B128,
    reserved_4: B96,
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
