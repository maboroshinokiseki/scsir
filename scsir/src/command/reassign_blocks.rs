#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReassignBlocksCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

pub struct ParameterBuilder<'a> {
    parent: &'a mut ReassignBlocksCommand<'a>,
    data_buffer: Vec<u8>,
    data_length: usize,
    long_lba_list: bool,
}

impl<'a> ReassignBlocksCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: vec![],
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&'a mut self) -> ParameterBuilder<'a> {
        ParameterBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl<'a> ParameterBuilder<'a> {
    fn new(parent: &'a mut ReassignBlocksCommand<'a>) -> Self {
        Self {
            parent,
            data_buffer: vec![],
            data_length: 0,
            long_lba_list: false,
        }
    }

    pub fn short_lba_list(&mut self, value: &[u32]) -> &mut Self {
        self.long_lba_list = false;
        self.data_buffer.clear();
        self.data_length = value.len() * size_of::<u32>();
        self.data_buffer
            .extend_from_slice(&(self.data_length as u32).to_be_bytes());
        for n in value {
            self.data_buffer.extend_from_slice(&n.to_be_bytes());
        }

        self
    }

    pub fn long_lba_list(&mut self, value: &[u64]) -> &mut Self {
        self.long_lba_list = true;
        self.data_buffer.clear();
        self.data_length = value.len() * size_of::<u64>();
        self.data_buffer
            .extend_from_slice(&(self.data_length as u32).to_be_bytes());
        for n in value {
            self.data_buffer.extend_from_slice(&n.to_be_bytes());
        }

        self
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut ReassignBlocksCommand> {
        let data_length_bits = if self.long_lba_list { 32 } else { 16 };

        bitfield_bound_check!(self.data_length, data_length_bits, "parameter length")?;

        self.parent
            .command_buffer
            .set_long_lba(self.long_lba_list.into());
        self.parent
            .command_buffer
            .set_long_list(self.long_lba_list.into());
        self.parent.data_buffer = std::mem::take(&mut self.data_buffer);
        Ok(self.parent)
    }
}

impl Scsi {
    pub fn reassign_blocks(&self) -> ReassignBlocksCommand {
        ReassignBlocksCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x07;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B6,
    long_lba: B1,
    long_list: B1,
    reserved_1: B24,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: VecBufferWrapper,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        self.data_buffer.clone()
    }

    fn data_size(&self) -> u32 {
        self.data_buffer.len() as u32
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

    const COMMAND_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
