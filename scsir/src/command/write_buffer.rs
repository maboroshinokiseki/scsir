#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReadBufferCommand<'a> {
    interface: &'a Scsi,
    mode_specific: u8,
    mode: u8,
    buffer_offset: u32,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl<'a> ReadBufferCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            mode_specific: 0,
            mode: 0,
            buffer_offset: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: vec![],
        }
    }

    // mode_specific must be less than 0x08
    pub fn mode_specific(&mut self, value: u8) -> &mut Self {
        self.mode_specific = value;
        self
    }

    // mode must be less than 0x20
    pub fn mode(&mut self, value: u8) -> &mut Self {
        self.mode = value;
        self
    }

    pub fn buffer_id(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_buffer_id(value);
        self
    }

    // buffer_offset must be less than 0x100_0000
    pub fn buffer_offset(&mut self, value: u32) -> &mut Self {
        self.buffer_offset = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.mode_specific, 3, "mode specific")?;
        bitfield_bound_check!(self.mode, 5, "mode")?;
        bitfield_bound_check!(self.buffer_offset, 24, "buffer offset")?;
        bitfield_bound_check!(self.data_buffer.len(), 24, "parameter length")?;

        let command_buffer = self
            .command_buffer
            .with_mode_specific(self.mode_specific)
            .with_mode(self.mode)
            .with_buffer_offset(self.buffer_offset)
            .with_parameter_list_length(self.data_buffer.len() as u32);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn write_buffer(&self) -> ReadBufferCommand {
        ReadBufferCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x3B;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    mode_specific: B3,
    mode: B5,
    buffer_id: B8,
    buffer_offset: B24,
    parameter_list_length: B24,
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

    const COMMAND_LENGTH: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
