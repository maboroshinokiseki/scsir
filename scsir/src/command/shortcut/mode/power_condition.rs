#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const POWER_CONDITION_PAGE_CODE: u8 = 0x1A;
pub const POWER_CONDITION_SUBPAGE_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PowerConditionPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
    pub pm_bg_precedence: B2,
    reserved_0: B5,
    pub standby_y: B1,
    reserved_1: B4,
    pub idle_c: B1,
    pub idle_b: B1,
    pub idle_a: B1,
    pub standby_z: B1,
    pub idle_a_condition_timer: B32,
    pub standby_z_condition_timer: B32,
    pub idle_b_condition_timer: B32,
    pub idle_c_condition_timer: B32,
    pub standby_y_condition_timer: B32,
    reserved_2: B120,
    pub check_condition_from_idle_c: B2,
    pub check_condition_from_standby: B2,
    pub check_condition_from_stopped: B2,
    reserved_3: B2,
}

impl ModePage for PowerConditionPage {
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

    const PAGE_LENGTH: usize = 40;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PowerConditionPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(PowerConditionPage))
        );
    }
}
