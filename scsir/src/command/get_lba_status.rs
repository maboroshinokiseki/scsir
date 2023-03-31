#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct GetLbaStatusCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
    descriptor_length: u32,
}

#[derive(Debug)]
pub struct CommandResult {
    pub total_descripter_length: usize,
    pub lba_status_descriptors: Vec<LbaStatusDescriptor>,
}

#[derive(Debug)]
pub struct LbaStatusDescriptor {
    pub logical_block_address: u64,
    pub number_of_logical_blocks: u32,
    pub provisioning_status: ProvisioningStatus,
}

#[derive(Debug)]
pub enum ProvisioningStatus {
    MappedOrUnknown,
    Deallocated,
    Anchored,
    Other(u8),
}

impl<'a> GetLbaStatusCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            descriptor_length: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
        }
    }

    pub fn starting_logical_block_address(&mut self, value: u64) -> &mut Self {
        self.command_buffer
            .set_starting_logical_block_address(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    // descriptor length must be less than 268435455(0xFFF_FFFF), which is (0xFFFF_FFFF - 8) / 16
    pub fn descriptor_length(&mut self, value: u32) -> &mut Self {
        self.descriptor_length = value;
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        const MAX_DESCRIPTOR_LENGTH: usize =
            (u32::MAX as usize - size_of::<ParameterHeader>()) / size_of::<Descriptor>();
        if self.descriptor_length > MAX_DESCRIPTOR_LENGTH as u32 {
            return Err(
                crate::Error::ArgumentOutOfBounds(
                    format!(
                        "descriptor length is out of bounds. The maximum possible value is {}, but {} was provided.",
                        MAX_DESCRIPTOR_LENGTH,
                        self.descriptor_length)));
        }

        let temp = ThisCommand {
            command_buffer: self.command_buffer.with_allocation_length(
                size_of::<ParameterHeader>() as u32
                    + self.descriptor_length * size_of::<Descriptor>() as u32,
            ),
            max_descriptor_length: self.descriptor_length,
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn get_lba_status(&self) -> GetLbaStatusCommand {
        GetLbaStatusCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x9E;
const SERVICE_ACTION: u8 = 0x12;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    starting_logical_block_address: B64,
    allocation_length: B32,
    reserved_1: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ParameterHeader {
    parameter_data_length: B32,
    reserved: B32,
}

#[bitfield]
#[derive(Clone, Copy)]
struct Descriptor {
    lba_status_logical_block_address: B64,
    number_oflogical_blocks: B32,
    provisioning_status: B8,
    reserved: B24,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    max_descriptor_length: u32,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = FlexibleStruct<ParameterHeader, Descriptor>;

    type ReturnType = crate::Result<CommandResult>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { FlexibleStruct::with_length(self.max_descriptor_length as usize) }
    }

    fn data_size(&self) -> u32 {
        self.max_descriptor_length * size_of::<Descriptor>() as u32
            + size_of::<ParameterHeader>() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let data = result.data;
        let length = unsafe { data.body_as_ref() }.parameter_data_length();
        let length = (length as usize - size_of::<u32>()) / size_of::<Descriptor>();

        let mut lba_status_descriptors = vec![];

        for item in unsafe { &data.elements_as_slice()[..usize::min(length, data.length())] } {
            let provisioning_status = match item.provisioning_status() {
                0 => ProvisioningStatus::MappedOrUnknown,
                1 => ProvisioningStatus::Deallocated,
                2 => ProvisioningStatus::Anchored,
                other => ProvisioningStatus::Other(other),
            };

            lba_status_descriptors.push(LbaStatusDescriptor {
                logical_block_address: item.lba_status_logical_block_address(),
                number_of_logical_blocks: item.number_oflogical_blocks(),
                provisioning_status,
            });
        }

        Ok(CommandResult {
            total_descripter_length: length,
            lba_status_descriptors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 16;
    const PARAMETER_HEADER_LENGTH: usize = 8;
    const DESCRIPTOR_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<ParameterHeader>(),
            PARAMETER_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ParameterHeader))
        );

        assert_eq!(
            size_of::<Descriptor>(),
            DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(Descriptor))
        );
    }
}
