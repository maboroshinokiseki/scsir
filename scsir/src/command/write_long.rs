#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct WriteLongCommand<'a> {
    interface: &'a Scsi,
    wr_uncor: bool,
    logical_block_address: u64,
    control: u8,
    data_buffer: Vec<u8>,
}

impl<'a> WriteLongCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            wr_uncor: false,
            logical_block_address: 0,
            control: 0,
            data_buffer: vec![],
        }
    }

    pub fn wr_uncor(&mut self, value: bool) -> &mut Self {
        self.wr_uncor = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u64) -> &mut Self {
        self.logical_block_address = value;
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

    pub fn issue_10(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.logical_block_address, 32, "logical block address")?;
        bitfield_bound_check!(self.data_buffer.len(), 16, "parameter length")?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_wr_uncor(self.wr_uncor.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_byte_transfer_length(self.data_buffer.len() as u16)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.data_buffer.len(), 16, "parameter length")?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_wr_uncor(self.wr_uncor.into())
            .with_service_action(SERVICE_ACTION_16)
            .with_logical_block_address(self.logical_block_address)
            .with_byte_transfer_length(self.data_buffer.len() as u16)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn write_long(&self) -> WriteLongCommand {
        WriteLongCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x3F;
const OPERATION_CODE_16: u8 = 0x9F;
const SERVICE_ACTION_16: u8 = 0x11;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    obsolete_0: B1,
    wr_uncor: B1,
    obsolete_1: B1,
    reserved_0: B4,
    obsolete_2: B1,
    logical_block_address: B32,
    reserved_1: B8,
    byte_transfer_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    obsolete_0: B1,
    wr_uncor: B1,
    obsolete_1: B1,
    service_action: B5,
    logical_block_address: B64,
    reserved_1: B16,
    byte_transfer_length: B16,
    reserved_2: B8,
    control: B8,
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
