#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct ReportTimestampCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

#[derive(Clone, Copy, Debug)]
pub struct CommandResult {
    pub timestamp_origin: u8,
    pub timestamp: u64,
}

impl<'a> ReportTimestampCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION)
                .with_allocation_length(size_of::<ReportTimestampParameterData>() as u32),
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn report_timestamp(&self) -> ReportTimestampCommand {
        ReportTimestampCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA3;
const SERVICE_ACTION: u8 = 0x0F;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    reserved_1: B32,
    allocation_length: B32,
    reserved_2: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReportTimestampParameterData {
    timestamp_parameter_data_length: B16,
    reserved_0: B5,
    timestamp_origin: B3,
    reserved_1: B8,
    timestamp: B48,
    reserved_2: B16,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = ReportTimestampParameterData;

    type DataBufferWrapper = ReportTimestampParameterData;

    type ReturnType = crate::Result<CommandResult>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        ReportTimestampParameterData::new()
    }

    fn data_size(&self) -> u32 {
        self.command_buffer.allocation_length()
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let data = result.data;
        Ok(CommandResult {
            timestamp_origin: data.timestamp_origin(),
            timestamp: data.timestamp(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 12;
    const PARAMETER_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<ReportTimestampParameterData>(),
            PARAMETER_LENGTH,
            concat!("Size of: ", stringify!(ReportTimestampParameterData))
        );
    }
}
