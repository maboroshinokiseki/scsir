#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::get_array,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReportLunsCommand<'a> {
    interface: &'a Scsi,
    descriptor_length: u32,
    command_buffer: CommandBuffer,
}

#[derive(Clone, Debug)]
pub struct CommandResult {
    pub total_descriptor_length: u32,
    pub descriptors: Vec<u64>,
}

impl<'a> ReportLunsCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            descriptor_length: 0,
        }
    }

    pub fn select_report(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_select_report(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    // descriptor length must be less than 536870910(0x1FFF_FFFE), which is (0xFFFF_FFFF - 8) / 8
    pub fn descriptor_length(&mut self, value: u32) -> &mut Self {
        self.descriptor_length = value;
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        let max_descriptor_length = (u32::MAX - 8) / 8;
        if self.descriptor_length > max_descriptor_length {
            return Err(
                crate::Error::ArgumentOutOfBounds(
                    format!(
                        "descriptor length is out of bounds. The maximum possible value is {}, but {} was provided.",
                        (u32::MAX - 8) / 8,
                        self.descriptor_length
            )));
        }

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_allocation_length(self.descriptor_length * 8 + 8),
        })
    }
}

impl Scsi {
    pub fn report_luns(&self) -> ReportLunsCommand {
        ReportLunsCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA0;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B8,
    select_report: B8,
    reserved_1: B24,
    allocation_length: B32,
    reserved_2: B8,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<CommandResult>;

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
        self.command_buffer.allocation_length()
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let (length, left) = get_array(result.data);
        let (skip, left) = get_array(left);
        _ = u32::from_be_bytes(skip);

        Ok(CommandResult {
            total_descriptor_length: u32::from_be_bytes(length),
            descriptors: left
                .chunks(size_of::<u64>())
                .map(|c| u64::from_be_bytes(get_array(c).0))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
