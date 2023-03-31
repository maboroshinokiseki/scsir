#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::sense::{SenseData, MAX_SENSE_BUFFER_LENGTH},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct RequestSenseCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

impl<'a> RequestSenseCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_allocation_length(MAX_SENSE_BUFFER_LENGTH as u8),
        }
    }

    pub fn descriptor_format(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_descriptor_format(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<SenseData> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn request_sense(&self) -> RequestSenseCommand {
        RequestSenseCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x03;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B7,
    descriptor_format: B1,
    reserved_1: B16,
    allocation_length: B8,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = [u8; MAX_SENSE_BUFFER_LENGTH];

    type DataBufferWrapper = [u8; MAX_SENSE_BUFFER_LENGTH];

    type ReturnType = crate::Result<SenseData>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        [0; MAX_SENSE_BUFFER_LENGTH]
    }

    fn data_size(&self) -> u32 {
        self.command_buffer.allocation_length() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(SenseData::parse(result.data, result.transfered_data_length))
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
