#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ChangeDefinitionCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    save: bool,
    definition_parameter: DefinitionParameter,
    control: u8,
    data_buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum DefinitionParameter {
    Current,
    Scsi2,
    Scsi3,
    ManufacturerDefault,
    Other(u8),
}

impl<'a> ChangeDefinitionCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            save: false,
            definition_parameter: DefinitionParameter::Current,
            control: 0,
            data_buffer: vec![],
        }
    }

    pub fn save(&mut self, value: bool) -> &mut Self {
        self.save = value;
        self
    }

    pub fn definition_parameter(&mut self, value: DefinitionParameter) -> &mut Self {
        self.definition_parameter = value;
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

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        let definition_parameter: u8 = self.definition_parameter.into();

        bitfield_bound_check!(definition_parameter, 7, "definition parameter")?;
        bitfield_bound_check!(self.data_buffer.len(), 8, "parameter data length")?;

        let temp = ThisCommand {
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_save(self.save as u8)
                .with_definition_parameter(definition_parameter)
                .with_parameter_list_length(self.data_buffer.len() as u8)
                .with_control(self.control),
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn change_definition(&self) -> ChangeDefinitionCommand<'_> {
        ChangeDefinitionCommand::new(self)
    }
}

impl From<DefinitionParameter> for u8 {
    fn from(value: DefinitionParameter) -> Self {
        match value {
            DefinitionParameter::Current => 0x00,
            DefinitionParameter::Scsi2 => 0x03,
            DefinitionParameter::Scsi3 => 0x04,
            DefinitionParameter::ManufacturerDefault => 0x3F,
            DefinitionParameter::Other(value) => value,
        }
    }
}

const OPERATION_CODE: u8 = 0x40;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B8,
    reserved_1: B7,
    save: B1,
    reserved_2: B1,
    definition_parameter: B7,
    reserved_3: B32,
    parameter_list_length: B8,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: VecBufferWrapper,
    timeout: Option<std::time::Duration>,
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

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
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
