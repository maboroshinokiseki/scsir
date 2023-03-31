#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct WriteSameCommand<'a> {
    interface: &'a Scsi,
    control: u8,
    stream_identifier: u16,
    group_number: u8,
    write_protect: u8,
    disable_page_out: bool,
    force_unit_access: bool,
    logical_block_address: u64,
    expected_initial_logical_block_reference_tag: u32,
    expected_logical_block_application_tag: u16,
    logical_block_application_tag_mask: u16,
    data_buffer: Vec<u8>,
    logical_block_size: u32,
}

impl<'a> WriteSameCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            control: 0,
            stream_identifier: 0,
            group_number: 0,
            write_protect: 0,
            disable_page_out: false,
            force_unit_access: false,
            logical_block_address: 0,
            expected_initial_logical_block_reference_tag: 0,
            expected_logical_block_application_tag: 0,
            logical_block_application_tag_mask: 0,
            data_buffer: vec![],
            logical_block_size: 512,
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn stream_identifier(&mut self, value: u16) -> &mut Self {
        self.stream_identifier = value;
        self
    }

    // group_number must be less than 0x40
    pub fn group_number(&mut self, value: u8) -> &mut Self {
        self.group_number = value;
        self
    }

    // write_protect must be less than 0x08
    pub fn write_protect(&mut self, value: u8) -> &mut Self {
        self.write_protect = value;
        self
    }

    pub fn disable_page_out(&mut self, value: bool) -> &mut Self {
        self.disable_page_out = value;
        self
    }

    pub fn force_unit_access(&mut self, value: bool) -> &mut Self {
        self.force_unit_access = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u64) -> &mut Self {
        self.logical_block_address = value;
        self
    }

    pub fn expected_initial_logical_block_reference_tag(&mut self, value: u32) -> &mut Self {
        self.expected_initial_logical_block_reference_tag = value;
        self
    }

    pub fn expected_logical_block_application_tag(&mut self, value: u16) -> &mut Self {
        self.expected_logical_block_application_tag = value;
        self
    }

    pub fn logical_block_application_tag_mask(&mut self, value: u16) -> &mut Self {
        self.logical_block_application_tag_mask = value;
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    fn error_check(&self, transfer_length_bits: u32, expect_tag: bool) -> crate::Result<()> {
        bitfield_bound_check!(self.group_number, 6, "group number")?;
        bitfield_bound_check!(self.write_protect, 3, "write protect")?;

        if self.data_buffer.len() % self.logical_block_size as usize != 0 {
            return Err(crate::Error::BadArgument(format!(
                "parameter length should be a multiple of logical block size, which is {}.",
                self.logical_block_size
            )));
        }

        if (self.data_buffer.len() / self.logical_block_size as usize)
            .checked_shr(transfer_length_bits)
            .unwrap_or_default()
            != 0
        {
            return Err(crate::Error::ArgumentOutOfBounds(format!(
                "parameter length is out of bounds. The maximum possible value is {}, but {} was provided.",
                1u128.wrapping_shl(transfer_length_bits) * self.logical_block_size as u128,
                self.data_buffer.len()
            )));
        }

        if !expect_tag
            && (self.expected_initial_logical_block_reference_tag != 0
                || self.expected_logical_block_application_tag != 0
                || self.logical_block_application_tag_mask != 0)
        {
            return Err(crate::Error::BadArgument(
                "expected tags and mask are not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn issue_16(&mut self) -> crate::Result<()> {
        self.error_check(16, false)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address)
            .with_stream_identifier(self.stream_identifier)
            .with_transfer_length(
                (self.data_buffer.len() / self.logical_block_size as usize) as u16,
            )
            .with_group_number(self.group_number)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }

    pub fn issue_32(&mut self) -> crate::Result<()> {
        self.error_check(32, true)?;

        let command_buffer = CommandBuffer32::new()
            .with_operation_code(OPERATION_CODE_32)
            .with_control(self.control)
            .with_stream_identifier(self.stream_identifier)
            .with_group_number(self.group_number)
            .with_additional_cdb_length(0x18)
            .with_service_action(SERVICE_ACTION_32)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address)
            .with_expected_initial_logical_block_reference_tag(
                self.expected_initial_logical_block_reference_tag,
            )
            .with_expected_logical_block_application_tag(
                self.expected_logical_block_application_tag,
            )
            .with_logical_block_application_tag_mask(self.logical_block_application_tag_mask)
            .with_transfer_length(
                (self.data_buffer.len() / self.logical_block_size as usize) as u32,
            );

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn write_stream(&self) -> WriteSameCommand {
        WriteSameCommand::new(self)
    }
}

const OPERATION_CODE_16: u8 = 0x9A;
const OPERATION_CODE_32: u8 = 0x7F;
const SERVICE_ACTION_32: u16 = 0x0010;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved_0: B3,
    logical_block_address: B64,
    stream_identifier: B16,
    transfer_length: B16,
    reserved_1: B2,
    group_number: B6,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer32 {
    operation_code: B8,
    control: B8,
    reserved_0: B16,
    stream_identifier: B16,
    reserved_1: B2,
    group_number: B6,
    additional_cdb_length: B8,
    service_action: B16,
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved_2: B3,
    reserved_3: B8,
    logical_block_address: B64,
    expected_initial_logical_block_reference_tag: B32,
    expected_logical_block_application_tag: B16,
    logical_block_application_tag_mask: B16,
    transfer_length: B32,
}

struct ThisCommand<C> {
    command_buffer: C,
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
        self.command_buffer
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

    const COMMAND_LENGTH_16: usize = 16;
    const COMMAND_LENGTH_32: usize = 32;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );

        assert_eq!(
            size_of::<CommandBuffer32>(),
            COMMAND_LENGTH_32,
            concat!("Size of: ", stringify!(CommandBuffer32))
        );
    }
}
