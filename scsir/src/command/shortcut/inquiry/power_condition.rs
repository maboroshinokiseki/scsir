#![allow(dead_code)]

use std::time::Duration;

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct PowerCondition {
    pub stopped_condition_recovery_time: Duration,
    pub standby_z_condition_recovery_time: Option<Duration>,
    pub standby_y_condition_recovery_time: Option<Duration>,
    pub idle_a_condition_recovery_time: Option<Duration>,
    pub idle_b_condition_recovery_time: Option<Duration>,
    pub idle_c_condition_recovery_time: Option<Duration>,
}

pub fn power_condition(this: &mut InquiryCommand) -> crate::Result<PowerCondition> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<Page, ()> = this.issue_flex(0)?;

    let body = result.get_body();

    let stopped_condition_recovery_time = match body.stopped_condition_recovery_time() {
        0x0000 | 0xFFFF => Duration::MAX,
        other => Duration::from_millis(other.into()),
    };

    fn convert_to_duration(valid: bool, duration: u16) -> Option<Duration> {
        match valid {
            true => Some(match duration {
                0x0000 | 0xFFFF => Duration::MAX,
                other => Duration::from_millis(other.into()),
            }),
            false => None,
        }
    }

    Ok(PowerCondition {
        stopped_condition_recovery_time,
        standby_z_condition_recovery_time: convert_to_duration(
            body.standby_z() != 0,
            body.standby_z_condition_recovery_time(),
        ),
        standby_y_condition_recovery_time: convert_to_duration(
            body.standby_y() != 0,
            body.standby_y_condition_recovery_time(),
        ),
        idle_a_condition_recovery_time: convert_to_duration(
            body.idle_a() != 0,
            body.idle_a_condition_recovery_time(),
        ),
        idle_b_condition_recovery_time: convert_to_duration(
            body.idle_b() != 0,
            body.idle_b_condition_recovery_time(),
        ),
        idle_c_condition_recovery_time: convert_to_duration(
            body.idle_c() != 0,
            body.idle_c_condition_recovery_time(),
        ),
    })
}

const PAGE_CODE: u8 = 0x8A;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct Page {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
    reserved_0: B6,
    standby_y: B1,
    standby_z: B1,
    reserved_1: B5,
    idle_c: B1,
    idle_b: B1,
    idle_a: B1,
    stopped_condition_recovery_time: B16,
    standby_z_condition_recovery_time: B16,
    standby_y_condition_recovery_time: B16,
    idle_a_condition_recovery_time: B16,
    idle_b_condition_recovery_time: B16,
    idle_c_condition_recovery_time: B16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_LENGTH: usize = 18;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<Page>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(Page))
        );
    }
}
