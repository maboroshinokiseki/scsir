#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct SanitizeCommand<'a> {
    interface: &'a Scsi,
    sanitize_service_action: ServiceAction,
    command_buffer: CommandBuffer,
    data_buffer: FlexibleStruct<OverwriteParameterListHeader, u8>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ServiceAction {
    Overwrite = 0x01,
    BlockErase = 0x02,
    CryptographicErase = 0x03,
    ExitFailureMode = 0x1F,
}

pub struct OverwriteParameterListBuilder<'a> {
    parent: &'a mut SanitizeCommand<'a>,
    test: u8,
    overwrite_count: u8,
    initialization_pattern: Vec<u8>,
    header_buffer: OverwriteParameterListHeader,
}

impl<'a> SanitizeCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            sanitize_service_action: ServiceAction::Overwrite,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(ServiceAction::Overwrite as u8),
            data_buffer: FlexibleStruct::new(),
        }
    }

    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_immediate(value.into());
        self
    }

    pub fn zoned_no_reset(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_zoned_no_reset(value.into());
        self
    }

    pub fn allow_unrestricted_sanitize_exit(&mut self, value: bool) -> &mut Self {
        self.command_buffer
            .set_allow_unrestricted_sanitize_exit(value.into());
        self
    }

    pub fn service_action(&mut self, value: ServiceAction) -> &mut Self {
        self.command_buffer.set_service_action(value as u8);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&'a mut self) -> OverwriteParameterListBuilder<'a> {
        OverwriteParameterListBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        if let ServiceAction::Overwrite = self.sanitize_service_action {
            self.command_buffer
                .set_parameter_list_length(self.data_buffer.total_size() as u16);
        }

        self.interface.issue(&ThisCommand {
            sanitize_service_action: self.sanitize_service_action,
            command_buffer: self.command_buffer,
            data_buffer: self.data_buffer.clone(),
        })
    }
}

impl<'a> OverwriteParameterListBuilder<'a> {
    fn new(parent: &'a mut SanitizeCommand<'a>) -> Self {
        Self {
            parent,
            test: 0,
            overwrite_count: 0,
            initialization_pattern: vec![],
            header_buffer: OverwriteParameterListHeader::new(),
        }
    }

    pub fn invert(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_invert(value.into());
        self
    }

    // test must be less than 0x02
    pub fn test(&mut self, value: u8) -> &mut Self {
        self.test = value;
        self
    }

    // overwrite_count must be less than 0x20
    pub fn overwrite_count(&mut self, value: u8) -> &mut Self {
        self.overwrite_count = value;
        self
    }

    // initialization_pattern length must be less than 0xFFFF
    pub fn initialization_pattern(&mut self, value: &[u8]) -> &mut Self {
        self.initialization_pattern = value.to_vec();
        self
    }

    pub fn done(&mut self) -> crate::Result<&'a mut SanitizeCommand> {
        bitfield_bound_check!(self.test, 2, "test")?;
        bitfield_bound_check!(self.overwrite_count, 5, "overwrite count")?;
        bitfield_bound_check!(
            self.initialization_pattern.len(),
            16,
            "initialization pattern length"
        )?;

        self.parent.data_buffer.set_body(
            self.header_buffer
                .with_test(self.test)
                .with_overwrite_count(self.overwrite_count)
                .with_initialization_pattern_length(self.initialization_pattern.len() as u16),
        );
        self.parent.data_buffer.clear();
        for n in &self.initialization_pattern {
            self.parent.data_buffer.push(*n)
        }

        Ok(self.parent)
    }
}

impl Scsi {
    pub fn sanitize(&self) -> SanitizeCommand {
        SanitizeCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x43;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    immediate: B1,
    zoned_no_reset: B1,
    allow_unrestricted_sanitize_exit: B1,
    service_action: B5,
    reserved: B40,
    parameter_list_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug, Default)]
struct OverwriteParameterListHeader {
    invert: B1,
    test: B2,
    overwrite_count: B5,
    reserved: B8,
    initialization_pattern_length: B16,
}

struct ThisCommand {
    sanitize_service_action: ServiceAction,
    command_buffer: CommandBuffer,
    data_buffer: FlexibleStruct<OverwriteParameterListHeader, u8>,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = FlexibleStruct<OverwriteParameterListHeader, u8>;

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
        match self.sanitize_service_action {
            ServiceAction::Overwrite => self.data_buffer.total_size() as u32,
            _ => 0,
        }
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
    const PARAMETER_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<OverwriteParameterListHeader>(),
            PARAMETER_HEADER_LENGTH,
            concat!("Size of: ", stringify!(OverwriteParameterListHeader))
        );
    }
}
