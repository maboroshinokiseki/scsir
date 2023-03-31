#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct SendDiagnosticCommand<'a> {
    interface: &'a Scsi,
    self_test_code: u8,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl<'a> SendDiagnosticCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            self_test_code: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: vec![],
        }
    }

    // self_test_code must be less than 0x08
    pub fn self_test_code(&mut self, value: u8) -> &mut Self {
        self.self_test_code = value;
        self
    }

    pub fn page_format(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_page_format(value.into());
        self
    }

    pub fn self_test(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_self_test(value.into());
        self
    }

    pub fn scsi_target_device_offline(&mut self, value: bool) -> &mut Self {
        self.command_buffer
            .set_scsi_target_device_offline(value.into());
        self
    }

    pub fn unit_offline(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_unit_offline(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer = value.to_owned();
        self.command_buffer
            .set_parameter_list_length(value.len() as u16);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.self_test_code, 3, "self test code")?;
        bitfield_bound_check!(self.data_buffer.len(), 16, "parameter list length")?;

        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer.with_self_test_code(self.self_test_code),
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn send_diagnostic(&self) -> SendDiagnosticCommand {
        SendDiagnosticCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x1D;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    self_test_code: B3,
    page_format: B1,
    reserved_0: B1,
    self_test: B1,
    scsi_target_device_offline: B1,
    unit_offline: B1,
    reserved: B8,
    parameter_list_length: B16,
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

    const COMMAND_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
