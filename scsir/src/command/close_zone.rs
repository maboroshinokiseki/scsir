#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct CloseZoneCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    command_buffer: CommandBuffer,
}

impl<'a> CloseZoneCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
        }
    }

    pub fn zone_id(&mut self, value: u64) -> &mut Self {
        self.command_buffer.set_zone_id(value);
        self
    }

    pub fn all(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_all(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
            timeout: self.timeout,
        })
    }
}

impl Scsi {
    pub fn close_zone(&self) -> CloseZoneCommand<'_> {
        CloseZoneCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x94;
const SERVICE_ACTION: u8 = 0x01;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    zone_id: B64,
    reserved_1: B32,
    reserved_2: B7,
    all: B1,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    timeout: Option<std::time::Duration>,
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

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
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
