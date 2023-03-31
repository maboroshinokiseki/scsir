#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct AsciiInformation {
    pub ascii_information: Vec<String>,
    pub vendor_information: Vec<u8>,
}

pub fn ascii_information(
    this: &mut InquiryCommand,
    page_code: u8,
) -> crate::Result<AsciiInformation> {
    if !(0x01..=0x7F).contains(&page_code) {
        return Err(crate::Error::ArgumentOutOfBounds(
            "ASCII Page code out of bounds".to_owned(),
        ));
    }

    this.page_code(Some(page_code));

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(0)?;
    let remaining = result.get_body().page_length() - 1;
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining as usize)?
    };

    let body = result.get_body();

    unsafe {
        Ok(AsciiInformation {
            ascii_information: result.elements_as_slice()[..body.ascii_length() as usize]
                .split(|c| *c == 0)
                .map(|s| String::from_utf8_lossy(s).to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            vendor_information: result.elements_as_slice()[body.ascii_length() as usize..]
                .to_owned(),
        })
    }
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    reserved: B8,
    page_length: B8,
    ascii_length: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 5;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );
    }
}
