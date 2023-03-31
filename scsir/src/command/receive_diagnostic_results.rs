#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReceiveDiagnosticResultsCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

impl<'a> ReceiveDiagnosticResultsCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
        }
    }

    pub fn page_code(&mut self, value: Option<u8>) -> &mut Self {
        self.command_buffer.set_page_code(value.unwrap_or(0));
        self.command_buffer
            .set_page_code_valid(value.is_some() as u8);
        self
    }

    pub fn allocation_length(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_allocation_length(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<Vec<u8>> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn receive_diagnostic_results(&self) -> ReceiveDiagnosticResultsCommand {
        ReceiveDiagnosticResultsCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x1C;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved: B7,
    page_code_valid: B1,
    page_code: B8,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<Vec<u8>>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { VecBufferWrapper::with_len(self.command_buffer.allocation_length() as usize) }
    }

    fn data_size(&self) -> u32 {
        self.command_buffer.allocation_length() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(std::mem::take(result.data))
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
