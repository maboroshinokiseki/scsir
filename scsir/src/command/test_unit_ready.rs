#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct TestUnitReadyCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

impl<'a> TestUnitReadyCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn test_unit_ready(&self) -> TestUnitReadyCommand {
        TestUnitReadyCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x00;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved: B32,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

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
