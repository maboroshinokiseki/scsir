#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct BackgroundControlCommand<'a> {
    interface: &'a Scsi,
    background_operation_control: u8,
    command_buffer: CommandBuffer,
}

impl<'a> BackgroundControlCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            background_operation_control: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
        }
    }

    // code must be one of the following values:
    // 0x0 Do not change host initiated advanced background operations.
    // 0x1 Start host initiated advanced background operations.
    // 0x2 Stop host initiated advanced background operations.
    // 0x3 Reserved
    pub fn background_operation_control(&mut self, value: u8) -> &mut Self {
        self.background_operation_control = value;
        self
    }

    // units of 100ms
    pub fn background_operation_time(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_background_operation_time(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(
            self.background_operation_control,
            2,
            "background operation control"
        )?;

        let temp = ThisCommand {
            command_buffer: self
                .command_buffer
                .with_background_operation_control(self.background_operation_control),
        };
        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn background_control(&self) -> BackgroundControlCommand {
        BackgroundControlCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x9E;
const SERVICE_ACTION: u8 = 0x15;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    background_operation_control: B2,
    reserved_1: B6,
    background_operation_time: B8,
    reserved_2: B88,
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

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
