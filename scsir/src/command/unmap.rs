#![allow(dead_code)]

use std::mem;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct UnmapCommand<'a> {
    interface: &'a Scsi,
    group_number: u8,
    command_buffer: CommandBuffer,
    data_buffer: FlexibleStruct<UnmapParameterHeader, UnmapBlockDescriptor>,
}

#[derive(Debug)]
pub struct ParameterBuilder<'a> {
    parent: &'a mut UnmapCommand<'a>,
    data_buffer: FlexibleStruct<UnmapParameterHeader, UnmapBlockDescriptor>,
}

impl<'a> UnmapCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            group_number: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: FlexibleStruct::new(),
        }
    }

    pub fn anchor(&mut self, anchor: bool) -> &mut Self {
        self.command_buffer.set_anchor(anchor as u8);
        self
    }

    // Group number must be less than 0x20
    pub fn group_number(&mut self, group_number: u8) -> &mut Self {
        self.group_number = group_number;
        self
    }

    pub fn control(&mut self, control: u8) -> &mut Self {
        self.command_buffer.set_control(control);
        self
    }

    pub fn parameter(&'a mut self) -> ParameterBuilder {
        ParameterBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.group_number, 5, "group number")?;

        let temp = ThisCommand {
            command_buffer: self.command_buffer,
            data_buffer: self.data_buffer.clone(),
        };
        self.interface.issue(&temp)
    }
}

impl<'a> ParameterBuilder<'a> {
    fn new(parent: &'a mut UnmapCommand<'a>) -> Self {
        Self {
            parent,
            data_buffer: FlexibleStruct::new(),
        }
    }

    // You shouldn't add more than 4095 descriptors, which is (2^16 - 1 - 8) / 16
    pub fn add_block_descriptor(
        &mut self,
        unmap_logical_block_address: u64,
        number_of_logical_blocks: u32,
    ) -> &mut Self {
        self.data_buffer.push(
            UnmapBlockDescriptor::new()
                .with_unmap_logical_block_address(unmap_logical_block_address)
                .with_number_of_logical_blocks(number_of_logical_blocks),
        );
        self
    }

    pub fn done(&mut self) -> crate::Result<&'a mut UnmapCommand> {
        let total_size = self.data_buffer.total_size();
        bitfield_bound_check!(total_size, 16, "parameter list length")?;

        self.parent
            .command_buffer
            .set_parameter_list_length(total_size as u16);
        let body = unsafe { self.data_buffer.body_as_mut() };
        body.set_unmap_data_length((total_size - mem::size_of::<u16>()) as u16);
        body.set_unmap_block_descriptor_data_length(
            (total_size - mem::size_of::<UnmapParameterHeader>()) as u16,
        );

        self.parent.data_buffer = std::mem::take(&mut self.data_buffer);
        Ok(self.parent)
    }
}

impl Scsi {
    pub fn unmap(&self) -> UnmapCommand {
        UnmapCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x42;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B7,
    anchor: B1,
    reserved_1: B32,
    reserved_2: B3,
    group_number: B5,
    parameter_list_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug, Default)]
struct UnmapParameterHeader {
    // size of unmap_block_descriptor_data_length + _reserved + unmap_block_descriptors
    unmap_data_length: B16,
    // size of unmap_block_descriptors
    unmap_block_descriptor_data_length: B16,
    reserved: B32,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct UnmapBlockDescriptor {
    unmap_logical_block_address: B64,
    number_of_logical_blocks: B32,
    reserved: B32,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: FlexibleStruct<UnmapParameterHeader, UnmapBlockDescriptor>,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = FlexibleStruct<UnmapParameterHeader, UnmapBlockDescriptor>;

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
        self.data_buffer.total_size() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const SG_UNMAP_CMD_LEN: usize = 10;
    const SG_UNMAP_PARAMETER_1_LEN: usize = 8 + 16;
    const SG_UNMAP_PARAMETER_2_LEN: usize = 8 + 16 * 2;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            SG_UNMAP_CMD_LEN,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        let mut temp = FlexibleStruct::<UnmapParameterHeader, UnmapBlockDescriptor>::new();
        let header = UnmapParameterHeader::new()
            .with_unmap_block_descriptor_data_length(0x0123)
            .with_unmap_data_length(0x4567);

        let block1 = UnmapBlockDescriptor::new()
            .with_number_of_logical_blocks(0x8998)
            .with_unmap_logical_block_address(0x7654);
        let block2 = UnmapBlockDescriptor::new()
            .with_number_of_logical_blocks(0x3210)
            .with_unmap_logical_block_address(0x0123);

        temp.set_body(header);

        temp.push(block1);

        assert_eq!(
            temp.total_size(),
            SG_UNMAP_PARAMETER_1_LEN,
            concat!("Size of UnmapParameterList with 1 element")
        );

        temp.push(block2);

        assert_eq!(
            temp.total_size(),
            SG_UNMAP_PARAMETER_2_LEN,
            concat!("Size of UnmapParameterList with 2 element")
        );

        assert_eq!(
            &temp.as_bytes()[..header.bytes.len()],
            &header.bytes,
            concat!("UnmapParameterHeader comparation")
        );

        assert_eq!(
            &temp.as_bytes()[header.bytes.len()..header.bytes.len() + block1.bytes.len()],
            &block1.bytes,
            concat!("UnmapBlockDescriptor 1 comparation")
        );

        assert_eq!(
            &temp.as_bytes()[header.bytes.len() + block1.bytes.len()
                ..header.bytes.len() + block1.bytes.len() + block2.bytes.len()],
            &block2.bytes,
            concat!("UnmapBlockDescriptor 2 comparation")
        );
    }
}
