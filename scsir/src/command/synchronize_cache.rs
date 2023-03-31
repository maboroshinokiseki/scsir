#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct SynchronizeCacheCommand<'a> {
    interface: &'a Scsi,
    immediate: bool,
    group_number: u8,
    logical_block_address: u64,
    number_of_blocks: u32,
    control: u8,
}

impl<'a> SynchronizeCacheCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            immediate: false,
            group_number: 0,
            logical_block_address: 0,
            number_of_blocks: 0,
            control: 0,
        }
    }

    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.immediate = value;
        self
    }

    // group_number must be less than 0x20
    pub fn group_number(&mut self, value: u8) -> &mut Self {
        self.group_number = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u64) -> &mut Self {
        self.logical_block_address = value;
        self
    }

    pub fn number_of_blocks(&mut self, value: u32) -> &mut Self {
        self.number_of_blocks = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    fn error_check(
        &self,
        logical_block_address_bits: u32,
        number_of_blocks_bits: u32,
    ) -> crate::Result<()> {
        bitfield_bound_check!(self.group_number, 5, "group number")?;
        bitfield_bound_check!(
            self.logical_block_address,
            logical_block_address_bits,
            "logical block address"
        )?;
        bitfield_bound_check!(
            self.number_of_blocks,
            number_of_blocks_bits,
            "number of blocks"
        )?;

        Ok(())
    }

    pub fn issue_10(&mut self) -> crate::Result<()> {
        self.error_check(32, 16)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_immediate(self.immediate.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_group_number(self.group_number)
            .with_number_of_blocks(self.number_of_blocks as u16)
            .with_control(self.control);

        self.interface.issue(&ThisCommand { command_buffer })
    }

    pub fn issue_16(&mut self) -> crate::Result<()> {
        self.error_check(64, 32)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_immediate(self.immediate.into())
            .with_logical_block_address(self.logical_block_address)
            .with_number_of_blocks(self.number_of_blocks)
            .with_group_number(self.group_number)
            .with_control(self.control);

        self.interface.issue(&ThisCommand { command_buffer })
    }
}

impl Scsi {
    pub fn synchronize_cache(&self) -> SynchronizeCacheCommand {
        SynchronizeCacheCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x35;
const OPERATION_CODE_16: u8 = 0x91;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B5,
    obsolete_0: B1,
    immediate: B1,
    obsolete_1: B1,
    logical_block_address: B32,
    reserved_1: B3,
    group_number: B5,
    number_of_blocks: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    reserved_0: B5,
    obsolete_0: B1,
    immediate: B1,
    obsolete_1: B1,
    logical_block_address: B64,
    number_of_blocks: B32,
    reserved_1: B3,
    group_number: B5,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;

    type DataBuffer = ();

    type DataBufferWrapper = ();

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::None
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {}

    fn data_size(&self) -> u32 {
        0
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_16: usize = 16;

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
    }
}
