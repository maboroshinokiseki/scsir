#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ModeSenseCommand<'a> {
    interface: &'a Scsi,
    long_lba_accepted: bool,
    disable_block_descriptors: bool,
    page_control: u8,
    page_code: u8,
    subpage_code: u8,
    allocation_length: u16,
    control: u8,
}

pub enum PageControl {
    Current,
    Changeable,
    Default,
    Saved,
}

impl<'a> ModeSenseCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            long_lba_accepted: false,
            disable_block_descriptors: false,
            page_control: 0,
            page_code: 0,
            subpage_code: 0,
            allocation_length: 0,
            control: 0,
        }
    }

    pub fn long_lba_accepted(&mut self, long_lba_accepted: bool) -> &mut Self {
        self.long_lba_accepted = long_lba_accepted;
        self
    }

    pub fn disable_block_descriptors(&mut self, disable_block_descriptors: bool) -> &mut Self {
        self.disable_block_descriptors = disable_block_descriptors;
        self
    }

    pub fn page_control(&mut self, page_control: PageControl) -> &mut Self {
        self.page_control = match page_control {
            PageControl::Current => 0b00,
            PageControl::Changeable => 0b01,
            PageControl::Default => 0b10,
            PageControl::Saved => 0b11,
        };
        self
    }

    // page_code must be less than 0x40
    pub fn page_code(&mut self, page_code: u8) -> &mut Self {
        self.page_code = page_code;
        self
    }

    pub fn subpage_code(&mut self, subpage_code: u8) -> &mut Self {
        self.subpage_code = subpage_code;
        self
    }

    pub fn allocation_length(&mut self, allocation_length: u16) -> &mut Self {
        self.allocation_length = allocation_length;
        self
    }

    pub fn control(&mut self, control: u8) -> &mut Self {
        self.control = control;
        self
    }

    fn error_check(
        &self,
        allocation_length_bits: u32,
        allow_long_lba_accepted: bool,
    ) -> crate::Result<()> {
        bitfield_bound_check!(self.page_control, 2, "page control")?;
        bitfield_bound_check!(self.page_code, 6, "page code")?;
        bitfield_bound_check!(
            self.allocation_length,
            allocation_length_bits,
            "allocation length"
        )?;

        if !allow_long_lba_accepted && self.long_lba_accepted {
            return Err(crate::Error::BadArgument(
                "long lba accepted is not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn issue_6(&mut self) -> crate::Result<Vec<u8>> {
        self.error_check(8, false)?;

        let command_buffer = CommandBuffer6::new()
            .with_operation_code(OPERATION_CODE_6)
            .with_disable_block_descriptors(self.disable_block_descriptors.into())
            .with_page_control(self.page_control)
            .with_page_code(self.page_code)
            .with_subpage_code(self.subpage_code)
            .with_allocation_length(self.allocation_length as u8)
            .with_control(self.control);

        let temp = ThisCommand {
            command_buffer,
            allocation_length: self.allocation_length.into(),
        };

        self.interface.issue(&temp)
    }

    pub fn issue_10(&mut self) -> crate::Result<Vec<u8>> {
        self.error_check(16, true)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_long_lba_accepted(self.long_lba_accepted.into())
            .with_disable_block_descriptors(self.disable_block_descriptors.into())
            .with_page_control(self.page_control)
            .with_page_code(self.page_code)
            .with_subpage_code(self.subpage_code)
            .with_allocation_length(self.allocation_length)
            .with_control(self.control);

        let temp = ThisCommand {
            command_buffer,
            allocation_length: self.allocation_length.into(),
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn mode_sense(&self) -> ModeSenseCommand {
        ModeSenseCommand::new(self)
    }
}

const OPERATION_CODE_6: u8 = 0x1A;
const OPERATION_CODE_10: u8 = 0x5A;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer6 {
    operation_code: B8,
    reserved_0: B4,
    disable_block_descriptors: B1,
    reserved_1: B3,
    page_control: B2,
    page_code: B6,
    subpage_code: B8,
    allocation_length: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B3,
    long_lba_accepted: B1,
    disable_block_descriptors: B1,
    reserved_1: B3,
    page_control: B2,
    page_code: B6,
    subpage_code: B8,
    reserved_2: B24,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
    allocation_length: usize,
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
        unsafe { VecBufferWrapper::with_len(self.allocation_length) }
    }

    fn data_size(&self) -> u32 {
        self.allocation_length as u32
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
