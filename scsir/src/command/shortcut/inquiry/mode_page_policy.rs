#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct ModePagePolicy {
    pub descriptors: Vec<ModePagePolicyDescriptor>,
}

#[derive(Debug)]
pub struct ModePagePolicyDescriptor {
    pub policy_page_code: u8,
    pub policy_subpage_code: u8,
    pub multiple_logical_units_share: bool,
    pub mode_page_policy: u8,
}

pub fn mode_page_policy(this: &mut InquiryCommand) -> crate::Result<ModePagePolicy> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, Descriptor> = this.issue_flex(0)?;
    let remaining = result.get_body().page_length() as usize / size_of::<Descriptor>();
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining)?
    };

    unsafe {
        Ok(ModePagePolicy {
            descriptors: result
                .elements_as_slice()
                .iter()
                .map(|e| ModePagePolicyDescriptor {
                    policy_page_code: e.policy_page_code(),
                    policy_subpage_code: e.policy_subpage_code(),
                    multiple_logical_units_share: e.multiple_logical_units_share() != 0,
                    mode_page_policy: e.mode_page_policy(),
                })
                .collect(),
        })
    }
}

const PAGE_CODE: u8 = 0x87;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Descriptor {
    reserved_0: B2,
    policy_page_code: B6,
    policy_subpage_code: B8,
    multiple_logical_units_share: B1,
    reserved_1: B5,
    mode_page_policy: B2,
    reserved_2: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;
    const DESCRIPTOR_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );

        assert_eq!(
            size_of::<Descriptor>(),
            DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(DescriptorHeader))
        );
    }
}
