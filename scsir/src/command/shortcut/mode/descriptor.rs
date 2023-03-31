#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ShortDescriptor {
    pub number_of_blocks: B32,
    reserved: B8,
    pub logical_block_length: B24,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct LongDescriptor {
    pub number_of_blocks: B64,
    reserved: B32,
    pub logical_block_length: B32,
}

#[derive(Clone, Copy, Debug)]
pub enum DescriptorType {
    Short,
    Long,
}

#[derive(Clone, Copy, Debug)]
pub enum DescriptorStorage {
    Short(ShortDescriptor),
    Long(LongDescriptor),
}

impl DescriptorStorage {
    pub fn from_bytes(descriptor_type: DescriptorType, bytes: &[u8]) -> (Self, &[u8]) {
        match descriptor_type {
            DescriptorType::Short => {
                let (array, bytes) = get_array(bytes);
                (
                    DescriptorStorage::Short(ShortDescriptor::from_bytes(array)),
                    bytes,
                )
            }
            DescriptorType::Long => {
                let (array, bytes) = get_array(bytes);
                (
                    DescriptorStorage::Long(LongDescriptor::from_bytes(array)),
                    bytes,
                )
            }
        }
    }

    pub fn number_of_blocks(&self) -> u64 {
        match self {
            DescriptorStorage::Short(d) => d.number_of_blocks() as u64,
            DescriptorStorage::Long(d) => d.number_of_blocks(),
        }
    }

    pub fn logical_block_length(&self) -> u32 {
        match self {
            DescriptorStorage::Short(d) => d.logical_block_length(),
            DescriptorStorage::Long(d) => d.logical_block_length(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            DescriptorStorage::Short(d) => d.bytes.to_vec(),
            DescriptorStorage::Long(d) => d.bytes.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SHORT_DESCRIPTOR_LENGTH: usize = 8;
    const LONG_DESCRIPTOR_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ShortDescriptor>(),
            SHORT_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(ShortDescriptor))
        );

        assert_eq!(
            size_of::<LongDescriptor>(),
            LONG_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(LongDescriptor))
        );
    }
}
