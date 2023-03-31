#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct ExtendedInquiryData {
    pub activate_microcode: u8,
    pub supported_protection_type: u8,
    pub guard_check: bool,
    pub application_tag_check: bool,
    pub reference_tag_check: bool,
    pub unit_attention_sense_key_supported: bool,
    pub grouping_function_supported: bool,
    pub priority_supported: bool,
    pub head_of_queue_supported: bool,
    pub ordered_supported: bool,
    pub simple_supported: bool,
    pub write_uncorrectable_supported: bool,
    pub correction_disable_supported: bool,
    pub non_volatile_cache_supported: bool,
    pub volatile_cache_supported: bool,
    pub no_protection_information_checking: bool,
    pub protection_information_interval_supported: bool,
    pub logical_unit_i_t_nexus_clear: bool,
    pub referrals_supported: bool,
    pub revert_to_defaults_bit_supported: bool,
    pub history_snapshots_release_effects: bool,
    pub multi_i_t_nexus_microcode_download: u8,
    pub extended_self_test_completion_minutes: u16,
    pub power_on_activation_supported: bool,
    pub hard_reset_activation_supported: bool,
    pub vendor_specific_activation_supported: bool,
    pub maximum_supported_sense_data_length: u8,
}

pub fn extended_inquiry_data(this: &mut InquiryCommand) -> crate::Result<ExtendedInquiryData> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    Ok(ExtendedInquiryData {
        activate_microcode: body.activate_microcode(),
        supported_protection_type: body.supported_protection_type(),
        guard_check: body.guard_check() != 0,
        application_tag_check: body.application_tag_check() != 0,
        reference_tag_check: body.reference_tag_check() != 0,
        unit_attention_sense_key_supported: body.unit_attention_sense_key_supported() != 0,
        grouping_function_supported: body.grouping_function_supported() != 0,
        priority_supported: body.priority_supported() != 0,
        head_of_queue_supported: body.head_of_queue_supported() != 0,
        ordered_supported: body.ordered_supported() != 0,
        simple_supported: body.simple_supported() != 0,
        write_uncorrectable_supported: body.write_uncorrectable_supported() != 0,
        correction_disable_supported: body.correction_disable_supported() != 0,
        non_volatile_cache_supported: body.non_volatile_cache_supported() != 0,
        volatile_cache_supported: body.volatile_cache_supported() != 0,
        no_protection_information_checking: body.no_protection_information_checking() != 0,
        protection_information_interval_supported: body.protection_information_interval_supported()
            != 0,
        logical_unit_i_t_nexus_clear: body.logical_unit_i_t_nexus_clear() != 0,
        referrals_supported: body.referrals_supported() != 0,
        revert_to_defaults_bit_supported: body.revert_to_defaults_bit_supported() != 0,
        history_snapshots_release_effects: body.history_snapshots_release_effects() != 0,
        multi_i_t_nexus_microcode_download: body.multi_i_t_nexus_microcode_download(),
        extended_self_test_completion_minutes: body.extended_self_test_completion_minutes(),
        power_on_activation_supported: body.power_on_activation_supported() != 0,
        hard_reset_activation_supported: body.hard_reset_activation_supported() != 0,
        vendor_specific_activation_supported: body.vendor_specific_activation_supported() != 0,
        maximum_supported_sense_data_length: body.maximum_supported_sense_data_length(),
    })
}

const PAGE_CODE: u8 = 0x86;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    activate_microcode: B2,
    supported_protection_type: B3,
    guard_check: B1,
    application_tag_check: B1,
    reference_tag_check: B1,
    reserved_00: B2,
    unit_attention_sense_key_supported: B1,
    grouping_function_supported: B1,
    priority_supported: B1,
    head_of_queue_supported: B1,
    ordered_supported: B1,
    simple_supported: B1,
    reserved_01: B4,
    write_uncorrectable_supported: B1,
    correction_disable_supported: B1,
    non_volatile_cache_supported: B1,
    volatile_cache_supported: B1,
    reserved_02: B2,
    no_protection_information_checking: B1,
    protection_information_interval_supported: B1,
    reserved_03: B3,
    logical_unit_i_t_nexus_clear: B1,
    reserved_04: B3,
    referrals_supported: B1,
    reserved_05: B1,
    revert_to_defaults_bit_supported: B1,
    history_snapshots_release_effects: B1,
    obsolete: B1,
    reserved_06: B4,
    multi_i_t_nexus_microcode_download: B4,
    extended_self_test_completion_minutes: B16,
    power_on_activation_supported: B1,
    hard_reset_activation_supported: B1,
    vendor_specific_activation_supported: B1,
    reserved_07: B5,
    maximum_supported_sense_data_length: B8,
    reserved_08: B128,
    reserved_09: B128,
    reserved_10: B128,
    reserved_11: B16,
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
