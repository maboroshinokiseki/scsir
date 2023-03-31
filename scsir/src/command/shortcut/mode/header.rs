#![allow(dead_code)]

use std::mem::size_of_val;

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ShortHeader {
    pub mode_data_length: B8,
    pub medium_type: B8,
    pub write_protect: B1,
    reserved_0: B2,
    pub dpo_and_fua_support: B1,
    reserved_1: B4,
    pub block_descriptor_length: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LongHeader {
    pub mode_data_length: B16,
    pub medium_type: B8,
    pub write_protect: B1,
    reserved_0: B2,
    pub dpo_and_fua_support: B1,
    reserved_1: B4,
    reserved_2: B7,
    pub long_lba: B1,
    reserved_3: B8,
    pub block_descriptor_length: B16,
}

#[derive(Clone, Copy, Debug)]
pub enum HeaderType {
    Short,
    Long,
}

#[derive(Clone, Copy, Debug)]
pub enum HeaderStorage {
    Short(ShortHeader),
    Long(LongHeader),
}

impl HeaderStorage {
    pub fn from_bytes(header_type: HeaderType, bytes: &[u8]) -> (Self, &[u8]) {
        match header_type {
            HeaderType::Short => {
                let (array, bytes) = get_array(bytes);
                (HeaderStorage::Short(ShortHeader::from_bytes(array)), bytes)
            }
            HeaderType::Long => {
                let (array, bytes) = get_array(bytes);
                (HeaderStorage::Long(LongHeader::from_bytes(array)), bytes)
            }
        }
    }

    pub fn required_allocation_length(&self) -> u16 {
        match self {
            HeaderStorage::Short(h) => (h.mode_data_length() as u16)
                .saturating_add(size_of_val(&h.mode_data_length()) as u16),
            HeaderStorage::Long(h) => h
                .mode_data_length()
                .saturating_add(size_of_val(&h.mode_data_length()) as u16),
        }
    }

    pub fn mode_data_length(&self) -> u16 {
        match self {
            HeaderStorage::Short(h) => h.mode_data_length() as u16,
            HeaderStorage::Long(h) => h.mode_data_length(),
        }
    }

    pub fn medium_type(&self) -> u8 {
        match self {
            HeaderStorage::Short(h) => h.medium_type(),
            HeaderStorage::Long(h) => h.medium_type(),
        }
    }

    pub fn write_protect(&self) -> bool {
        match self {
            HeaderStorage::Short(h) => h.write_protect() != 0,
            HeaderStorage::Long(h) => h.write_protect() != 0,
        }
    }

    pub fn dpo_and_fua_support(&self) -> bool {
        match self {
            HeaderStorage::Short(h) => h.dpo_and_fua_support() != 0,
            HeaderStorage::Long(h) => h.dpo_and_fua_support() != 0,
        }
    }

    pub fn long_lba(&self) -> bool {
        match self {
            HeaderStorage::Short(_) => false,
            HeaderStorage::Long(h) => h.long_lba() != 0,
        }
    }

    pub fn block_descriptor_length(&self) -> u16 {
        match self {
            HeaderStorage::Short(h) => h.block_descriptor_length() as u16,
            HeaderStorage::Long(h) => h.block_descriptor_length(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            HeaderStorage::Short(h) => h.bytes.to_vec(),
            HeaderStorage::Long(h) => h.bytes.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SHORT_HEADER_LENGTH: usize = 4;
    const LONG_HEADER_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ShortHeader>(),
            SHORT_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ShortHeader))
        );

        assert_eq!(
            size_of::<LongHeader>(),
            LONG_HEADER_LENGTH,
            concat!("Size of: ", stringify!(LongHeader))
        );
    }
}
