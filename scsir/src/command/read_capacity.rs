#![allow(dead_code)]

use std::{marker::PhantomData, mem::size_of};

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct ReadCapacityCommand<'a> {
    interface: &'a Scsi,
    control: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct ReadCapacity10Result {
    pub returned_logical_block_address: u32,
    pub block_length_in_bytes: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ReadCapacity16Result {
    pub returned_logical_block_address: u64,
    pub logical_block_length_in_bytes: u32,
    pub read_capacity_basis: u8,
    pub protection_type: u8,
    pub protection_enabled: bool,
    pub p_i_exponent: u8,
    pub logical_blocks_per_physical_block_exponent: u8,
    pub logical_block_provisioning_management_enabled: bool,
    pub logical_block_provisioning_read_zeros: bool,
    pub lowest_aligned_logical_block_address: u16,
}

impl<'a> ReadCapacityCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            control: 0,
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn issue_10(&mut self) -> crate::Result<ReadCapacity10Result> {
        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_control(self.control);

        let result = self.interface.issue(&ThisCommand {
            command_buffer,
            marker: PhantomData::<DataBuffer10>,
        })?;

        Ok(ReadCapacity10Result {
            returned_logical_block_address: result.returned_logical_block_address(),
            block_length_in_bytes: result.block_length_in_bytes(),
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<ReadCapacity16Result> {
        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_service_action(SERVICE_ACTION_16)
            .with_allocation_length(size_of::<DataBuffer16>() as u32)
            .with_control(self.control);

        let result = self.interface.issue(&ThisCommand {
            command_buffer,
            marker: PhantomData::<DataBuffer16>,
        })?;

        Ok(ReadCapacity16Result {
            returned_logical_block_address: result.returned_logical_block_address(),
            logical_block_length_in_bytes: result.logical_block_length_in_bytes(),
            read_capacity_basis: result.read_capacity_basis(),
            protection_type: result.protection_type(),
            protection_enabled: result.protection_enabled() != 0,
            p_i_exponent: result.p_i_exponent(),
            logical_blocks_per_physical_block_exponent: result
                .logical_blocks_per_physical_block_exponent(),
            logical_block_provisioning_management_enabled: result
                .logical_block_provisioning_management_enabled()
                != 0,
            logical_block_provisioning_read_zeros: result.logical_block_provisioning_read_zeros()
                != 0,
            lowest_aligned_logical_block_address: result.lowest_aligned_logical_block_address(),
        })
    }
}

impl Scsi {
    pub fn read_capacity(&self) -> ReadCapacityCommand {
        ReadCapacityCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x25;
const OPERATION_CODE_16: u8 = 0x9E;
const SERVICE_ACTION_16: u8 = 0x10;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B7,
    obsolete: B1,
    logical_block_address: B32,
    reserved_1: B16,
    reserved_2: B7,
    pmi: B1,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    logical_block_address: u64,
    allocation_length: B32,
    reserved_1: B7,
    pmi: B1,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct DataBuffer10 {
    returned_logical_block_address: B32,
    block_length_in_bytes: B32,
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct DataBuffer16 {
    returned_logical_block_address: B64,
    logical_block_length_in_bytes: B32,
    reserved_0: B2,
    read_capacity_basis: B2,
    protection_type: B3,
    protection_enabled: B1,
    p_i_exponent: B4,
    logical_blocks_per_physical_block_exponent: B4,
    logical_block_provisioning_management_enabled: B1,
    logical_block_provisioning_read_zeros: B1,
    lowest_aligned_logical_block_address: B14,
    reserved_1: B128,
}

struct ThisCommand<C, D> {
    command_buffer: C,

    marker: PhantomData<D>,
}

impl<C: Copy, D: Copy + Default> Command for ThisCommand<C, D> {
    type CommandBuffer = C;

    type DataBuffer = D;

    type DataBufferWrapper = D;

    type ReturnType = crate::Result<D>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        D::default()
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(*result.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_16: usize = 16;
    const DATA_LENGTH_10: usize = 8;
    const DATA_LENGTH_16: usize = 32;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer10>(),
            COMMAND_LENGTH_10,
            concat!("Size of: ", stringify!(CommandBuffer10))
        );

        assert_eq!(
            size_of::<CommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );

        assert_eq!(
            size_of::<DataBuffer10>(),
            DATA_LENGTH_10,
            concat!("Size of: ", stringify!(DataBuffer10))
        );

        assert_eq!(
            size_of::<DataBuffer16>(),
            DATA_LENGTH_16,
            concat!("Size of: ", stringify!(DataBuffer16))
        );
    }
}
