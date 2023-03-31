#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{command::bitfield_bound_check, result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct StartStopUnitCommand<'a> {
    interface: &'a Scsi,
    power_condition_modifer: u8,
    power_condition: u8,
    command_buffer: CommandBuffer,
}

impl<'a> StartStopUnitCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            power_condition_modifer: 0,
            power_condition: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
        }
    }

    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_immediate(value.into());
        self
    }

    // power_condition_modifer must be less than 0x10
    pub fn power_condition_modifer(&mut self, value: u8) -> &mut Self {
        self.power_condition_modifer = value;
        self
    }

    // power_condition must be less than 0x10
    pub fn power_condition(&mut self, value: u8) -> &mut Self {
        self.power_condition = value;
        self
    }

    pub fn no_flush(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_no_flush(value.into());
        self
    }

    pub fn load_eject(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_load_eject(value.into());
        self
    }

    pub fn start(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_start(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(self.power_condition_modifer, 4, "power condition modifer")?;
        bitfield_bound_check!(self.power_condition, 4, "power condition")?;

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_power_condition_modifer(self.power_condition_modifer)
                .with_power_condition(self.power_condition),
        })
    }
}

impl Scsi {
    pub fn start_stop_unit(&self) -> StartStopUnitCommand {
        StartStopUnitCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x1B;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B7,
    immediate: B1,
    reserved_1: B8,
    reserved_2: B4,
    power_condition_modifer: B4,
    power_condition: B4,
    reserved_3: B1,
    no_flush: B1,
    load_eject: B1,
    start: B1,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
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
