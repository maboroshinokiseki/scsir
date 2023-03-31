#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ModeSelectCommand<'a> {
    interface: &'a Scsi,
    page_format: bool,
    revert_to_defaults: bool,
    saved_pages: bool,
    control: u8,
    data_buffer: Vec<u8>,
}

impl<'a> ModeSelectCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            page_format: false,
            revert_to_defaults: false,
            saved_pages: false,
            control: 0,
            data_buffer: vec![],
        }
    }

    pub fn page_format(&mut self, value: bool) -> &mut Self {
        self.page_format = value;
        self
    }

    pub fn revert_to_defaults(&mut self, value: bool) -> &mut Self {
        self.revert_to_defaults = value;
        self
    }

    pub fn saved_pages(&mut self, value: bool) -> &mut Self {
        self.saved_pages = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    fn error_check(
        &self,
        parameter_length_bits: u32,
        allow_revert_to_defaults: bool,
    ) -> crate::Result<()> {
        bitfield_bound_check!(
            self.data_buffer.len(),
            parameter_length_bits,
            "parameter length"
        )?;

        if !allow_revert_to_defaults && self.revert_to_defaults {
            return Err(crate::Error::BadArgument(
                "revert to defaults is not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn issue_6(&mut self) -> crate::Result<()> {
        self.error_check(8, true)?;

        let temp = ThisCommand {
            command: CommandBuffer6::new()
                .with_operation_code(OPERATION_CODE_6)
                .with_page_format(self.page_format.into())
                .with_revert_to_defaults(self.revert_to_defaults.into())
                .with_saved_pages(self.saved_pages.into())
                .with_parameter_list_length(self.data_buffer.len() as u8)
                .with_control(self.control),
            data_buffer: self.data_buffer.clone().into(),
        };

        self.interface.issue(&temp)
    }

    pub fn issue_10(&mut self) -> crate::Result<()> {
        self.error_check(16, false)?;

        let temp = ThisCommand {
            command: CommandBuffer10::new()
                .with_operation_code(OPERATION_CODE_10)
                .with_page_format(self.page_format.into())
                .with_saved_pages(self.saved_pages.into())
                .with_parameter_list_length(self.data_buffer.len() as u16)
                .with_control(self.control),
            data_buffer: self.data_buffer.clone().into(),
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn mode_select(&self) -> ModeSelectCommand {
        ModeSelectCommand::new(self)
    }
}

const OPERATION_CODE_6: u8 = 0x15;
const OPERATION_CODE_10: u8 = 0x55;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer6 {
    operation_code: B8,
    reserved_0: B3,
    page_format: B1,
    reserved_1: B2,
    revert_to_defaults: B1,
    saved_pages: B1,
    reserved_2: B16,
    parameter_list_length: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B3,
    page_format: B1,
    reserved_1: B3,
    saved_pages: B1,
    reserved_2: B40,
    parameter_list_length: B16,
    control: B8,
}

struct ThisCommand<C: Copy> {
    command: C,
    data_buffer: VecBufferWrapper,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::ToDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command
    }

    fn data(&self) -> Self::DataBufferWrapper {
        self.data_buffer.clone()
    }

    fn data_size(&self) -> u32 {
        self.data_buffer.len() as u32
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

    const COMMAND_LENGTH_6: usize = 6;
    const COMMAND_LENGTH_10: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer6>(),
            COMMAND_LENGTH_6,
            concat!("Size of: ", stringify!(CommandBuffer6))
        );

        assert_eq!(
            size_of::<CommandBuffer10>(),
            COMMAND_LENGTH_10,
            concat!("Size of: ", stringify!(CommandBuffer10))
        );
    }
}
