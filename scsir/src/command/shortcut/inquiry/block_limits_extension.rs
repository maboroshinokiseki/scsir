#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct BlockLimitsExtension {
    pub maximum_number_of_streams: u16,
    pub optimal_stream_write_size: u16,
    pub stream_granularity_size: u32,
    pub additional_data: Vec<u8>,
}

pub fn block_limits_extension(this: &mut InquiryCommand) -> crate::Result<BlockLimitsExtension> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(0)?;

    let remaining = result.get_body().page_length();
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining as usize)?
    };

    let body = result.get_body();

    Ok(BlockLimitsExtension {
        maximum_number_of_streams: body.maximum_number_of_streams(),
        optimal_stream_write_size: body.optimal_stream_write_size(),
        stream_granularity_size: body.stream_granularity_size(),
        additional_data: unsafe { result.elements_as_slice().to_vec() },
    })
}

const PAGE_CODE: u8 = 0xB7;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved: B16,
    maximum_number_of_streams: B16,
    optimal_stream_write_size: B16,
    stream_granularity_size: B32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 14;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );
    }
}
