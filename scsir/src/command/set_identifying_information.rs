#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct SetIdentifyingInformationCommand<'a> {
    interface: &'a Scsi,
    information_type: u8,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl<'a> SetIdentifyingInformationCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            information_type: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
            data_buffer: vec![],
        }
    }

    // information_type must be less than 0x80
    pub fn information_type(&mut self, value: u8) -> &mut Self {
        self.information_type = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer = value.to_owned();
        self.command_buffer
            .set_parameter_list_length(value.len() as u32);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.information_type, 7, "information type")?;
        bitfield_bound_check!(self.data_buffer.len(), 32, "parameter list length")?;

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_information_type(self.information_type),
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn set_identifying_information(&self) -> SetIdentifyingInformationCommand {
        SetIdentifyingInformationCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA4;
const SERVICE_ACTION: u8 = 0x06;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    reserved_1: B32,
    parameter_list_length: B32,
    information_type: B7,
    reserved_2: B1,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: VecBufferWrapper,
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
