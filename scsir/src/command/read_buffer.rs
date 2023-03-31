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
    buffer_offset: u64,
    allocation_length: u32,
    buffer_id: u8,
    control: u8,
}

impl<'a> ReadBufferCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            mode_specific: 0,
            mode: 0,
            buffer_offset: 0,
            allocation_length: 0,
            buffer_id: 0,
            control: 0,
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

    // buffer_offset must be less than 0xFF_FFFF for issue_10
    pub fn buffer_offset(&mut self, value: u64) -> &mut Self {
        self.buffer_offset = value;
        self
    }

    // allocation_length must be less than 0xFF_FFFF for issue_10
    pub fn allocation_length(&mut self, value: u32) -> &mut Self {
        self.allocation_length = value;
        self
    }

    pub fn buffer_id(&mut self, value: u8) -> &mut Self {
        self.buffer_id = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    fn error_check(
        &self,
        buffer_offset_bits: u32,
        allocation_length_bits: u32,
    ) -> crate::Result<()> {
        bitfield_bound_check!(self.mode_specific, 3, "mode specific")?;
        bitfield_bound_check!(self.mode, 5, "mode")?;
        bitfield_bound_check!(self.buffer_offset, buffer_offset_bits, "buffer offset")?;
        bitfield_bound_check!(
            self.allocation_length,
            allocation_length_bits,
            "allocation length"
        )?;

        Ok(())
    }

    pub fn issue_10(&mut self) -> crate::Result<Vec<u8>> {
        self.error_check(24, 24)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_mode_specific(self.mode_specific)
            .with_mode(self.mode)
            .with_buffer_id(self.buffer_id)
            .with_buffer_offset(self.buffer_offset as u32)
            .with_allocation_length(self.allocation_length)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length: self.allocation_length,
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<Vec<u8>> {
        self.error_check(64, 32)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_mode_specific(self.mode_specific)
            .with_mode(self.mode)
            .with_buffer_offset(self.buffer_offset)
            .with_allocation_length(self.allocation_length)
            .with_buffer_id(self.buffer_id)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length: self.allocation_length,
        })
    }
}

impl Scsi {
    pub fn read_buffer(&self) -> ReadBufferCommand {
        ReadBufferCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x3C;
const OPERATION_CODE_16: u8 = 0x9B;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    mode_specific: B3,
    mode: B5,
    buffer_id: B8,
    buffer_offset: B24,
    allocation_length: B24,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    mode_specific: B3,
    mode: B5,
    buffer_offset: B64,
    allocation_length: B32,
    buffer_id: B8,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
    allocation_length: u32,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;

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
        unsafe { VecBufferWrapper::with_len(self.allocation_length as usize) }
    }

    fn data_size(&self) -> u32 {
        self.allocation_length
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(std::mem::take(result.data).0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_16: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer10>(),
            COMMAND_LENGTH_10,
            concat!("Size of: ", stringify!(CommandBuffer10))
        );

        assert_eq!(
            size_of::<CommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );
    }
}
