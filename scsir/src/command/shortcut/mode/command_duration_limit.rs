#![allow(dead_code)]
#![allow(unused_braces)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

pub const COMMAND_DURATION_LIMIT_PAGE_CODE: u8 = 0x0A;
pub const COMMAND_DURATION_LIMIT_A_SUBPAGE_CODE: u8 = 0x03;
pub const COMMAND_DURATION_LIMIT_B_SUBPAGE_CODE: u8 = 0x04;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct CommandDurationLimitPage {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
    reserved: B32,
    pub descriptor_1: CommandDurationLimitDescriptor,
    pub descriptor_2: CommandDurationLimitDescriptor,
    pub descriptor_3: CommandDurationLimitDescriptor,
    pub descriptor_4: CommandDurationLimitDescriptor,
    pub descriptor_5: CommandDurationLimitDescriptor,
    pub descriptor_6: CommandDurationLimitDescriptor,
    pub descriptor_7: CommandDurationLimitDescriptor,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
pub struct CommandDurationLimitDescriptor {
    pub cdlunit: B3,
    reserved: B13,
    pub command_duration_limit: B16,
}

impl ModePage for CommandDurationLimitPage {
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

    const PAGE_LENGTH: usize = 36;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandDurationLimitPage>(),
            PAGE_LENGTH,
            concat!("Size of: ", stringify!(CommandDurationLimitPage))
        );
    }
}
