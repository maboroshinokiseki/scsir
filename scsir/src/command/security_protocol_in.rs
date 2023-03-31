#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct SecurityProtocolInCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

impl<'a> SecurityProtocolInCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
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

    pub fn allocation_length(&mut self, value: u32) -> &mut Self {
        self.command_buffer.set_allocation_length(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<Vec<u8>> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn security_protocol_in(&self) -> SecurityProtocolInCommand {
        SecurityProtocolInCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA2;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    security_protocol: B8,
    security_protocol_specific: B16,
    inc_512: B1,
    reserved_0: B7,
    reserved_1: B8,
    allocation_length: B32,
    reserved_2: B8,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

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
        let allocation_length = if self.command_buffer.inc_512() == 0 {
            self.command_buffer.allocation_length() as usize
        } else {
            (self.command_buffer.allocation_length() as usize).saturating_mul(512)
        };

        unsafe { VecBufferWrapper::with_len(allocation_length) }
    }

    fn data_size(&self) -> u32 {
        if self.command_buffer.inc_512() == 0 {
            self.command_buffer.allocation_length()
        } else {
            self.command_buffer.allocation_length().saturating_mul(512)
        }
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
