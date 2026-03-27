#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReadLongCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    pblock: bool,
    corrct: bool,
    logical_block_address: u64,
    byte_transfer_length: u16,
    control: u8,
}

impl<'a> ReadLongCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            pblock: false,
            corrct: false,
            logical_block_address: 0,
            byte_transfer_length: 0,
            control: 0,
        }
    }

    pub fn physical_block(&mut self, value: bool) -> &mut Self {
        self.pblock = value;
        self
    }

    pub fn correct(&mut self, value: bool) -> &mut Self {
        self.corrct = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u64) -> &mut Self {
        self.logical_block_address = value;
        self
    }

    pub fn byte_transfer_length(&mut self, value: u16) -> &mut Self {
        self.byte_transfer_length = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn issue_10(&mut self) -> crate::Result<Vec<u8>> {
        bitfield_bound_check!(self.logical_block_address, 32, "logical block address")?;

        let temp = ThisCommand {
            command_buffer: CommandBuffer10::new()
                .with_operation_code(OPERATION_CODE_10)
                .with_pblock(self.pblock as u8)
                .with_corrct(self.corrct as u8)
                .with_logical_block_address(self.logical_block_address as u32)
                .with_byte_transfer_length(self.byte_transfer_length)
                .with_control(self.control),
            allocation_length: self.byte_transfer_length as u32,
            timeout: self.timeout,
        };

        self.interface.issue(&temp)
    }

    pub fn issue_16(&mut self) -> crate::Result<Vec<u8>> {
        let temp = ThisCommand {
            command_buffer: CommandBuffer16::new()
                .with_operation_code(OPERATION_CODE_16)
                .with_service_action(SERVICE_ACTION_16)
                .with_logical_block_address(self.logical_block_address)
                .with_byte_transfer_length(self.byte_transfer_length)
                .with_pblock(self.pblock as u8)
                .with_corrct(self.corrct as u8)
                .with_control(self.control),
            allocation_length: self.byte_transfer_length as u32,
            timeout: self.timeout,
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn read_long(&self) -> ReadLongCommand<'_> {
        ReadLongCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x3E;
const OPERATION_CODE_16: u8 = 0x9E;
const SERVICE_ACTION_16: u8 = 0x11;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B5,
    pblock: B1,
    corrct: B1,
    obsolete: B1,
    logical_block_address: B32,
    reserved_1: B8,
    byte_transfer_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer16 {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    logical_block_address: B64,
    reserved_1: B16,
    byte_transfer_length: B16,
    reserved_2: B6,
    pblock: B1,
    corrct: B1,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
    allocation_length: u32,
    timeout: Option<std::time::Duration>,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;

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
        unsafe { VecBufferWrapper::with_len(self.allocation_length as usize) }
    }

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
    }

    fn data_size(&self) -> u32 {
        self.allocation_length
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(std::mem::take(result.data).0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_16: usize = 16;

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
    }
}
