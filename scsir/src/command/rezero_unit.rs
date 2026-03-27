#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct RezeroUnitCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    lun: u8,
    control: u8,
}

impl<'a> RezeroUnitCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            lun: 0,
            control: 0,
        }
    }

    pub fn lun(&mut self, value: u8) -> &mut Self {
        self.lun = value;
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

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.lun, 3, "lun")?;

        self.interface.issue(&ThisCommand {
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_lun(self.lun)
                .with_control(self.control),
            timeout: self.timeout,
        })
    }
}

impl Scsi {
    pub fn rezero_unit(&self) -> RezeroUnitCommand<'_> {
        RezeroUnitCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x01;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    lun: B3,
    logical_block_address: B21,
    reserved: B8,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    timeout: Option<std::time::Duration>,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

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
