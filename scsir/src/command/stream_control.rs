#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct StreamControlCommand<'a> {
    interface: &'a Scsi,
    stream_control: u8,
    command_buffer: CommandBuffer,
    data_buffer: DataBuffer,
}

#[derive(Debug)]
pub struct ParameterBuilder<'a> {
    parent: &'a mut StreamControlCommand<'a>,
    data_buffer: DataBuffer,
}

impl<'a> StreamControlCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            stream_control: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
            data_buffer: DataBuffer::new(),
        }
    }

    // stream_control must be less than 0x0x04
    pub fn stream_control(&mut self, value: u8) -> &mut Self {
        self.stream_control = value;
        self
    }

    pub fn stream_identifier(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_stream_identifier(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&'a mut self) -> ParameterBuilder<'a> {
        ParameterBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.stream_control, 2, "stream control")?;

        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer.with_stream_control(self.stream_control),
            data_buffer: self.data_buffer,
        })
    }
}

impl<'a> ParameterBuilder<'a> {
    fn new(parent: &'a mut StreamControlCommand<'a>) -> Self {
        Self {
            parent,
            data_buffer: DataBuffer::new().with_parameter_length(0x07),
        }
    }

    pub fn assigned_stream_identifier(&mut self, value: u16) -> &mut Self {
        self.data_buffer.set_assigned_stream_identifier(value);
        self
    }

    pub fn done(&mut self) -> crate::Result<&'a mut StreamControlCommand> {
        self.parent.data_buffer = self.data_buffer;
        Ok(self.parent)
    }
}

impl Scsi {
    pub fn stream_control(&self) -> StreamControlCommand {
        StreamControlCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x9E;
const SERVICE_ACTION: u8 = 0x14;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B1,
    stream_control: B2,
    service_action: B5,
    reserved_1: B16,
    stream_identifier: B16,
    reserved_2: B72,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct DataBuffer {
    parameter_length: B8,
    reserved_0: B24,
    assigned_stream_identifier: B16,
    reserved_1: B16,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: DataBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = DataBuffer;

    type DataBufferWrapper = DataBuffer;

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::None
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        self.data_buffer
    }

    fn data_size(&self) -> u32 {
        size_of::<DataBuffer>() as u32
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

    const COMMAND_LENGTH: usize = 16;
    const DATA_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<DataBuffer>(),
            DATA_LENGTH,
            concat!("Size of: ", stringify!(DataBuffer))
        );
    }
}
