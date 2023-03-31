#![allow(dead_code)]

use std::{mem::size_of, slice};

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct LogSelectCommand<'a> {
    interface: &'a Scsi,
    page_control: u8,
    page_code: u8,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl<'a> LogSelectCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            page_control: 0,
            page_code: 0,
            data_buffer: vec![],
        }
    }

    pub fn parameter_code_reset(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_parameter_code_reset(value as u8);
        self
    }

    pub fn save_parameters(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_save_parameters(value as u8);
        self
    }

    // page_control must be less than 0x04
    pub fn page_control(&mut self, value: u8) -> &mut Self {
        self.page_control = value;
        self
    }

    // page_code must be less than 0x40
    pub fn page_code(&mut self, value: u8) -> &mut Self {
        self.page_code = value;
        self
    }

    pub fn subpage_code(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_subpage_code(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    // parameter length must be less or equal than 0xFFFF
    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.page_control, 2, "page control")?;
        bitfield_bound_check!(self.page_code, 6, "page code")?;
        bitfield_bound_check!(self.data_buffer.len(), 16, "parameter list length")?;

        let temp = ThisCommand {
            command_buffer: self
                .command_buffer
                .with_page_control(self.page_control)
                .with_page_code(self.page_code)
                .with_parameter_list_length(self.data_buffer.len() as u16),
            parameter: self.data_buffer.clone().into(),
        };

        self.interface.issue(&temp)?;

        Ok(())
    }

    pub fn issue_generic<T: Copy>(&mut self, parameter: T) -> crate::Result<()> {
        let u8_slice: &[u8] =
            unsafe { slice::from_raw_parts(&parameter as *const _ as *const _, size_of::<T>()) };
        self.parameter(u8_slice);

        self.issue()
    }
}

impl Scsi {
    pub fn log_select(&self) -> LogSelectCommand {
        LogSelectCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x4C;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B6,
    parameter_code_reset: B1,
    save_parameters: B1,
    page_control: B2,
    page_code: B6,
    subpage_code: B8,
    reserved_1: B24,
    parameter_list_length: B16,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    parameter: VecBufferWrapper,
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
        self.parameter.clone()
    }

    fn data_size(&self) -> u32 {
        self.parameter.len() as u32
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
