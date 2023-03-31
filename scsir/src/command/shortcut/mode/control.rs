#![allow(dead_code)]
#![allow(deprecated)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const CONTROL_PAGE_CODE: u8 = 0x0A;
pub const CONTROL_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ControlPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub task_set_type: B3,
    pub task_management_functions_only: B1,
    pub dpicz: B1,
    pub descriptor_format_sense_data: B1,
    pub global_logging_target_save_disable: B1,
    pub report_log_exception_condition: B1,
    pub queue_algorithm_modifier: B4,
    pub no_unit_attention_on_release: B1,
    pub queue_error_management: B2,
    #[deprecated]
    pub disable_queuing: B1,
    pub vendor_specific: B1,
    pub report_a_check: B1,
    pub unit_attention_interlocks_control: B2,
    pub software_write_protect: B1,
    #[deprecated]
    pub ready_aer_permission: B1,
    #[deprecated]
    pub unit_attention_aer_permission: B1,
    #[deprecated]
    pub error_aer_permission: B1,
    pub application_tag_owner: B1,
    pub task_aborted_status: B1,
    pub application_tag_mode_page_enabled: B1,
    pub reject_write_without_protection: B1,
    reserved: B1,
    pub autoload_mode: B3,
    #[deprecated]
    pub ready_aer_holdoff_period: B16,
    pub busy_timeout_period: B16,
    pub extended_self_test_completion_time: B16,
}

impl ModePage for ControlPage {
    fn new() -> Self {
        Self::new()
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);

        (Self::from_bytes(array), bytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ControlPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(ControlPage))
        );
    }
}
