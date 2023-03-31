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
    group_number: u8,
    write_protect: u8,
    anchor: bool,
    unmap: bool,
    no_data_out_buffer: bool,
    logical_block_address: u64,
    expected_initial_logical_block_reference_tag: u32,
    expected_logical_block_application_tag: u16,
    logical_block_application_tag_mask: u16,
    number_of_blocks: u32,
    data_buffer: Vec<u8>,
}

impl<'a> WriteSameCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            control: 0,
            group_number: 0,
            write_protect: 0,
            anchor: false,
            unmap: false,
            no_data_out_buffer: false,
            logical_block_address: 0,
            expected_initial_logical_block_reference_tag: 0,
            expected_logical_block_application_tag: 0,
            logical_block_application_tag_mask: 0,
            number_of_blocks: 0,
            data_buffer: vec![],
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    // group_number must be less than 0x20
    pub fn group_number(&mut self, value: u8) -> &mut Self {
        self.group_number = value;
        self
    }

    // write_protect must be less than 0x08
    pub fn write_protect(&mut self, value: u8) -> &mut Self {
        self.write_protect = value;
        self
    }

    pub fn anchor(&mut self, value: bool) -> &mut Self {
        self.anchor = value;
        self
    }

    pub fn unmap(&mut self, value: bool) -> &mut Self {
        self.unmap = value;
        self
    }

    pub fn no_data_out_buffer(&mut self, value: bool) -> &mut Self {
        self.no_data_out_buffer = value;
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

    pub fn number_of_blocks(&mut self, value: u32) -> &mut Self {
        self.number_of_blocks = value;
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    fn error_check(
        &self,
        logical_block_address_bits: u32,
        number_of_blocks_bits: u32,
        allow_no_data_out_buffer: bool,
        expect_tag: bool,
    ) -> crate::Result<()> {
        bitfield_bound_check!(self.group_number, 5, "group number")?;
        bitfield_bound_check!(self.write_protect, 3, "write protect")?;
        bitfield_bound_check!(
            self.logical_block_address,
            logical_block_address_bits,
            "logical block address"
        )?;
        bitfield_bound_check!(
            self.number_of_blocks,
            number_of_blocks_bits,
            "number of blocks length"
        )?;

        if !allow_no_data_out_buffer && self.no_data_out_buffer {
            return Err(crate::Error::BadArgument(
                "no data out buffer is not allowed here".to_owned(),
            ));
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

    pub fn issue_10(&mut self) -> crate::Result<()> {
        self.error_check(32, 16, false, false)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_write_protect(self.write_protect)
            .with_anchor(self.anchor.into())
            .with_unmap(self.unmap.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_group_number(self.group_number)
            .with_number_of_blocks(self.number_of_blocks as u16)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<()> {
        self.error_check(64, 32, true, false)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_write_protect(self.write_protect)
            .with_anchor(self.anchor.into())
            .with_unmap(self.unmap.into())
            .with_no_data_out_buffer(self.no_data_out_buffer.into())
            .with_logical_block_address(self.logical_block_address)
            .with_number_of_blocks(self.number_of_blocks)
            .with_group_number(self.group_number)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }

    pub fn issue_32(&mut self) -> crate::Result<()> {
        self.error_check(64, 32, true, true)?;

        let command_buffer = CommandBuffer32::new()
            .with_operation_code(OPERATION_CODE_32)
            .with_control(self.control)
            .with_group_number(self.group_number)
            .with_additional_cdb_length(0x18)
            .with_service_action(SERVICE_ACTION_32)
            .with_write_protect(self.write_protect)
            .with_anchor(self.anchor.into())
            .with_unmap(self.unmap.into())
            .with_no_data_out_buffer(self.no_data_out_buffer.into())
            .with_logical_block_address(self.logical_block_address)
            .with_expected_initial_logical_block_reference_tag(
                self.expected_initial_logical_block_reference_tag,
            )
            .with_expected_logical_block_application_tag(
                self.expected_logical_block_application_tag,
            )
            .with_logical_block_application_tag_mask(self.logical_block_application_tag_mask)
            .with_number_of_blocks(self.number_of_blocks);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn write_same(&self) -> WriteSameCommand {
        WriteSameCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x41;
const OPERATION_CODE_16: u8 = 0x93;
const OPERATION_CODE_32: u8 = 0x7F;
const SERVICE_ACTION_32: u16 = 0x000D;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    write_protect: B3,
    anchor: B1,
    unmap: B1,
    obsolete: B3,
    logical_block_address: B32,
    reserved: B3,
    group_number: B5,
    number_of_blocks: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    write_protect: B3,
    anchor: B1,
    unmap: B1,
    obsolete: B2,
    no_data_out_buffer: B1,
    logical_block_address: B64,
    number_of_blocks: B32,
    reserved: B3,
    group_number: B5,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer32 {
    operation_code: B8,
    control: B8,
    reserved_0: B32,
    reserved_1: B3,
    group_number: B5,
    additional_cdb_length: B8,
    service_action: B16,
    write_protect: B3,
    anchor: B1,
    unmap: B1,
    obsolete: B2,
    no_data_out_buffer: B1,
    reserved_2: B8,
    logical_block_address: B64,
    expected_initial_logical_block_reference_tag: B32,
    expected_logical_block_application_tag: B16,
    logical_block_application_tag_mask: B16,
    number_of_blocks: B32,
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

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_16: usize = 16;
    const COMMAND_LENGTH_32: usize = 32;

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

        assert_eq!(
            size_of::<CommandBuffer32>(),
            COMMAND_LENGTH_32,
            concat!("Size of: ", stringify!(CommandBuffer32))
        );
    }
}
