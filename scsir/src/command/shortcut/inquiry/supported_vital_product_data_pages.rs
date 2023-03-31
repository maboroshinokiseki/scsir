#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct SupportedVitalProductDataPages {
    pub supported_pages: Vec<u8>,
}

pub fn supported_vital_product_data_pages(
    this: &mut InquiryCommand,
) -> crate::Result<SupportedVitalProductDataPages> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(0)?;
    let remaining = result.get_body().page_length();
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining as usize)?
    };

    Ok(SupportedVitalProductDataPages {
        supported_pages: unsafe { Vec::from(result.elements_as_slice()) },
    })
}

const PAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    reserved: B8,
    page_length: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );
    }
}
