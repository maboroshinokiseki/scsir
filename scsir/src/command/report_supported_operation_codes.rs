#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::{bitfield_bound_check, get_array},
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReportSupportedOperationCodesCommand<'a> {
    interface: &'a Scsi,
    reporting_options: u8,
    command_buffer: CommandBuffer,
}

#[derive(Clone, Debug)]
pub enum CommandResult {
    AllCommands(AllCommands),
    OneCommand(OneCommand),
    Other(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct AllCommands {
    pub required_allocation_length: u32,
    pub descriptors: Vec<CommandDescriptor>,
}

#[derive(Clone, Debug)]
pub struct CommandDescriptor {
    pub operation_code: u8,
    pub service_action: Option<u16>,
    pub cdb_length: u16,
    pub timeout_descriptor: Option<TimeoutsDescriptor>,
}

#[derive(Clone, Debug)]
pub struct OneCommand {
    pub support: u8,
    pub cdb_usage_data: Vec<u8>,
    pub timeout_descriptor: Option<TimeoutsDescriptor>,
}

#[derive(Clone, Debug)]
pub struct TimeoutsDescriptor {
    pub command_specific: u8,
    pub nominal_command_processing_timeout: u32,
    pub recommend_command_timeout: u32,
}

impl<'a> ReportSupportedOperationCodesCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            reporting_options: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
        }
    }

    pub fn return_command_timeouts_descriptor(&mut self, value: bool) -> &mut Self {
        self.command_buffer
            .set_return_command_timeouts_descriptor(value.into());
        self
    }

    // reporting_options must be less than 0x08
    pub fn reporting_options(&mut self, value: u8) -> &mut Self {
        self.reporting_options = value;
        self
    }

    pub fn requested_operation_code(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_requested_operation_code(value);
        self
    }

    pub fn requested_service_action(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_requested_service_action(value);
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

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        bitfield_bound_check!(self.reporting_options, 3, "reporting options")?;

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_reporting_options(self.reporting_options),
        })
    }
}

impl Scsi {
    pub fn report_supported_operation_codes(&self) -> ReportSupportedOperationCodesCommand {
        ReportSupportedOperationCodesCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA3;
const SERVICE_ACTION: u8 = 0x0C;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    return_command_timeouts_descriptor: B1,
    reserved_1: B4,
    reporting_options: B3,
    requested_operation_code: B8,
    requested_service_action: B16,
    allocation_length: B32,
    reserved_2: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct AllCommandsParameterDataHeader {
    command_data_length: B32,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandDescriptorHeader {
    operation_code: B8,
    reserved_0: B8,
    service_action: B16,
    reserved_1: B8,
    reserved_2: B6,
    command_timeouts_descriptor_present: B1,
    service_action_valid: B1,
    cdb_length: B16,
}

#[bitfield]
#[derive(Clone, Copy)]
struct OneCommandParameterDataHeader {
    reserved_0: B8,
    command_timeout_descriptor_present: B1,
    reserved_1: B4,
    support: B3,
    cdb_size: B16,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandTimeoutsDescriptor {
    descriptor_length: B16,
    reserved: B8,
    command_specific: B8,
    nominal_command_processing_timeout: B32,
    recommend_command_timeout: B32,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<CommandResult>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { VecBufferWrapper::with_len(self.command_buffer.allocation_length() as usize) }
    }

    fn data_size(&self) -> u32 {
        self.command_buffer.allocation_length()
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        if self.command_buffer.reporting_options() == 0 {
            let (bytes, left) = get_array(result.data);
            let header = AllCommandsParameterDataHeader::from_bytes(bytes);
            let mut descriptors = vec![];
            let mut left = &left[..usize::min(header.command_data_length() as usize, left.len())];
            while !left.is_empty() {
                let (bytes, l) = get_array(left);
                left = l;
                let command_descriptor = CommandDescriptorHeader::from_bytes(bytes);
                let service_action = if command_descriptor.service_action_valid() != 0 {
                    Some(command_descriptor.service_action())
                } else {
                    None
                };

                let timeout_descriptor = if command_descriptor.command_timeouts_descriptor_present()
                    != 0
                {
                    let (bytes, l) = get_array(left);
                    left = l;
                    let timeout_descriptor = CommandTimeoutsDescriptor::from_bytes(bytes);
                    Some(TimeoutsDescriptor {
                        command_specific: timeout_descriptor.command_specific(),
                        nominal_command_processing_timeout: timeout_descriptor
                            .nominal_command_processing_timeout(),
                        recommend_command_timeout: timeout_descriptor.recommend_command_timeout(),
                    })
                } else {
                    None
                };

                descriptors.push(CommandDescriptor {
                    operation_code: command_descriptor.operation_code(),
                    service_action,
                    cdb_length: command_descriptor.cdb_length(),
                    timeout_descriptor,
                })
            }

            Ok(CommandResult::AllCommands(AllCommands {
                required_allocation_length: header.command_data_length()
                    + size_of::<AllCommandsParameterDataHeader>() as u32,
                descriptors,
            }))
        } else if self.command_buffer.reporting_options() < 0b100 {
            let (bytes, left) = get_array(result.data);
            let header = OneCommandParameterDataHeader::from_bytes(bytes);
            let (cdb_data, left) =
                left[..].split_at(usize::min(header.cdb_size() as usize, left.len()));
            let cdb_data = Vec::from(cdb_data);
            let timeout_descriptor = if header.command_timeout_descriptor_present() != 0 {
                let (bytes, _) = get_array(left);
                let timeout_descriptor = CommandTimeoutsDescriptor::from_bytes(bytes);
                Some(TimeoutsDescriptor {
                    command_specific: timeout_descriptor.command_specific(),
                    nominal_command_processing_timeout: timeout_descriptor
                        .nominal_command_processing_timeout(),
                    recommend_command_timeout: timeout_descriptor.recommend_command_timeout(),
                })
            } else {
                None
            };

            Ok(CommandResult::OneCommand(OneCommand {
                support: header.support(),
                cdb_usage_data: cdb_data,
                timeout_descriptor,
            }))
        } else {
            Ok(CommandResult::Other(std::mem::take(result.data)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 12;
    const ALL_COMMANDS_PARAMETER_DATA_HEADER_LENGTH: usize = 4;
    const COMMAND_DESCRIPTOR_HEADER_LENGTH: usize = 8;
    const ONE_COMMAND_PARAMETER_DATA_HEADER_LENGTH: usize = 4;
    const COMMAND_TIMEOUTS_DESCRIPTOR_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<AllCommandsParameterDataHeader>(),
            ALL_COMMANDS_PARAMETER_DATA_HEADER_LENGTH,
            concat!("Size of: ", stringify!(AllCommandsParameterDataHeader))
        );

        assert_eq!(
            size_of::<CommandDescriptorHeader>(),
            COMMAND_DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(CommandDescriptorHeader))
        );

        assert_eq!(
            size_of::<OneCommandParameterDataHeader>(),
            ONE_COMMAND_PARAMETER_DATA_HEADER_LENGTH,
            concat!("Size of: ", stringify!(OneCommandParameterDataHeader))
        );

        assert_eq!(
            size_of::<CommandTimeoutsDescriptor>(),
            COMMAND_TIMEOUTS_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(CommandTimeoutsDescriptor))
        );
    }
}
