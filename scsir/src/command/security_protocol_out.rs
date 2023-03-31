#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct SecurityProtocolOutCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl<'a> SecurityProtocolOutCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: vec![],
        }
    }

    pub fn security_protocol(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_security_protocol(value);
        self
    }

    pub fn security_protocol_specific(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_security_protocol_specific(value);
        self
    }

    pub fn inc_512(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_inc_512(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer = value.to_owned();
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        let transfer_length = if self.command_buffer.inc_512() == 0 {
            self.data_buffer.len()
        } else {
            self.data_buffer.len() / 512
        };

        bitfield_bound_check!(transfer_length, 32, "parameter length")?;

        if self.command_buffer.inc_512() == 1 && self.data_buffer.len() % 512 != 0 {
            return Err(crate::Error::BadArgument(
                "parameter length is not a multiple of 512".to_owned(),
            ));
        }

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_transfer_length(transfer_length as u32),
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn security_protocol_out(&self) -> SecurityProtocolOutCommand {
        SecurityProtocolOutCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xB5;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    security_protocol: B8,
    security_protocol_specific: B16,
    inc_512: B1,
    reserved_0: B7,
    reserved_1: B8,
    transfer_length: B32,
    reserved_2: B8,
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
