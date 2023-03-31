#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct BlockLimits {
    pub write_same_non_zero: bool,
    pub maximum_compare_and_write_length: u8,
    pub optimal_transfer_length_granularity: u16,
    pub maximum_transfer_length: u32,
    pub optimal_transfer_length: u32,
    pub maximum_prefetch_length: u32,
    pub maximum_unmap_lba_count: u32,
    pub maximum_unmap_block_descriptor_count: u32,
    pub optimal_unmap_granularity: u32,
    pub unmap_granularity_alignment: Option<u32>,
    pub maximum_write_same_length: u64,
    pub maximum_atomic_transfer_length: u32,
    pub atomic_alignment: u32,
    pub atomic_transfer_length_granularity: u32,
    pub maximum_atomic_transfer_length_with_atomic_boundary: u32,
    pub maximum_atomic_boundary_size: u32,
}

pub fn block_limits(this: &mut InquiryCommand) -> crate::Result<BlockLimits> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    let unmap_granularity_alignment = match body.unmap_granularity_alignment_valid() != 0 {
        true => Some(body.unmap_granularity_alignment()),
        false => None,
    };

    Ok(BlockLimits {
        write_same_non_zero: body.write_same_non_zero() != 0,
        maximum_compare_and_write_length: body.maximum_compare_and_write_length(),
        optimal_transfer_length_granularity: body.optimal_transfer_length_granularity(),
        maximum_transfer_length: body.maximum_transfer_length(),
        optimal_transfer_length: body.optimal_transfer_length(),
        maximum_prefetch_length: body.maximum_prefetch_length(),
        maximum_unmap_lba_count: body.maximum_unmap_lba_count(),
        maximum_unmap_block_descriptor_count: body.maximum_unmap_block_descriptor_count(),
        optimal_unmap_granularity: body.optimal_unmap_granularity(),
        unmap_granularity_alignment,
        maximum_write_same_length: body.maximum_write_same_length(),
        maximum_atomic_transfer_length: body.maximum_atomic_transfer_length(),
        atomic_alignment: body.atomic_alignment(),
        atomic_transfer_length_granularity: body.atomic_transfer_length_granularity(),
        maximum_atomic_transfer_length_with_atomic_boundary: body
            .maximum_atomic_transfer_length_with_atomic_boundary(),
        maximum_atomic_boundary_size: body.maximum_atomic_boundary_size(),
    })
}

const PAGE_CODE: u8 = 0xB0;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved: B7,
    write_same_non_zero: B1,
    maximum_compare_and_write_length: B8,
    optimal_transfer_length_granularity: B16,
    maximum_transfer_length: B32,
    optimal_transfer_length: B32,
    maximum_prefetch_length: B32,
    maximum_unmap_lba_count: B32,
    maximum_unmap_block_descriptor_count: B32,
    optimal_unmap_granularity: B32,
    unmap_granularity_alignment_valid: B1,
    unmap_granularity_alignment: B31,
    maximum_write_same_length: B64,
    maximum_atomic_transfer_length: B32,
    atomic_alignment: B32,
    atomic_transfer_length_granularity: B32,
    maximum_atomic_transfer_length_with_atomic_boundary: B32,
    maximum_atomic_boundary_size: B32,
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
