#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct SeekCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    lun: u8,
    logical_block_address: u32,
    control: u8,
}

impl<'a> SeekCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            lun: 0,
            logical_block_address: 0,
            control: 0,
        }
    }

    pub fn lun(&mut self, value: u8) -> &mut Self {
        self.lun = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u32) -> &mut Self {
        self.logical_block_address = value;
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

    pub fn issue_6(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.lun, 3, "lun")?;
        bitfield_bound_check!(self.logical_block_address, 21, "logical block address")?;

        let temp = ThisCommand {
            command_buffer: CommandBuffer6::new()
                .with_operation_code(OPERATION_CODE_6)
                .with_lun(self.lun)
                .with_logical_block_address(self.logical_block_address)
                .with_control(self.control),
            timeout: self.timeout,
        };

        self.interface.issue(&temp)
    }

    pub fn issue_10(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.lun, 3, "lun")?;

        let temp = ThisCommand {
            command_buffer: CommandBuffer10::new()
                .with_operation_code(OPERATION_CODE_10)
                .with_lun(self.lun)
                .with_logical_block_address(self.logical_block_address)
                .with_control(self.control),
            timeout: self.timeout,
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn seek(&self) -> SeekCommand<'_> {
        SeekCommand::new(self)
    }
}

const OPERATION_CODE_6: u8 = 0x0B;
const OPERATION_CODE_10: u8 = 0x2B;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer6 {
    operation_code: B8,
    lun: B3,
    logical_block_address: B21,
    reserved_1: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer10 {
    operation_code: B8,
    lun: B3,
    reserved_0: B5,
    logical_block_address: B32,
    reserved_1: B24,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
    timeout: Option<std::time::Duration>,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;

    type DataBuffer = ();

    type DataBufferWrapper = ();

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::None
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
    }

    fn data(&self) -> Self::DataBufferWrapper {}

    fn data_size(&self) -> u32 {
        0
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

    const COMMAND_LENGTH_6: usize = 6;
    const COMMAND_LENGTH_10: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer6>(),
            COMMAND_LENGTH_6,
            concat!("Size of: ", stringify!(CommandBuffer6))
        );

        assert_eq!(
            size_of::<CommandBuffer10>(),
            COMMAND_LENGTH_10,
            concat!("Size of: ", stringify!(CommandBuffer10))
        );
    }
}
