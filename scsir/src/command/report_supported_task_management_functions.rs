#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{result_data::ResultData, Command, DataDirection, Scsi};

#[derive(Clone, Debug)]
pub struct ReportSupportedTaskManagementFunctionsCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

#[derive(Clone, Copy, Debug)]
pub struct CommandResult {
    pub abort_task_supported: bool,
    pub abort_task_set_supported: bool,
    pub clear_aca_supported: bool,
    pub clear_task_set_supported: bool,
    pub logical_unit_reset_supported: bool,
    pub query_task_supported: bool,
    pub query_asynchronous_event_supported: bool,
    pub query_task_set_supported: bool,
    pub i_t_nexus_reset_supported: bool,
    pub task_management_function_timeouts_valid: bool,
    pub abort_task_timeout_selector: bool,
    pub abort_task_set_timeout_selector: bool,
    pub clear_aca_timeout_selector: bool,
    pub clear_task_set_timeout_selector: bool,
    pub logical_unit_reset_timeout_selector: bool,
    pub query_task_timeout_selector: bool,
    pub query_asynchronous_event_timeout_selector: bool,
    pub query_task_set_timeout_selector: bool,
    pub i_t_nexus_reset_timeout_selector: bool,
    pub task_management_functions_long_timeout: u32,
    pub task_management_functions_short_timeout: u32,
}

impl<'a> ReportSupportedTaskManagementFunctionsCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION)
                .with_allocation_length(size_of::<
                    ReportSupportedTaskManagementFunctionsExtendedParameterData,
                >() as u32),
        }
    }

    pub fn return_extended_parameter_data(&mut self, value: bool) -> &mut Self {
        self.command_buffer
            .set_return_extended_parameter_data(value.into());
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        self.interface.issue(&ThisCommand {
            command_buffer: self.command_buffer,
        })
    }
}

impl Scsi {
    pub fn report_supported_task_management_functions(
        &self,
    ) -> ReportSupportedTaskManagementFunctionsCommand {
        ReportSupportedTaskManagementFunctionsCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0xA3;
const SERVICE_ACTION: u8 = 0x0D;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    return_extended_parameter_data: B1,
    reserved_1: B31,
    allocation_length: B32,
    reserved_2: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReportSupportedTaskManagementFunctionsExtendedParameterData {
    abort_task_supported: B1,
    abort_task_set_supported: B1,
    clear_aca_supported: B1,
    clear_task_set_supported: B1,
    logical_unit_reset_supported: B1,
    query_task_supported: B1,
    obsolete: B2,
    reserved_0: B5,
    query_asynchronous_event_supported: B1,
    query_task_set_supported: B1,
    i_t_nexus_reset_supported: B1,
    reserved_1: B8,
    report_supported_task_management_functions_additional_data_length: B8,
    reserved_2: B7,
    task_management_function_timeouts_valid: B1,
    reserved_3: B8,
    abort_task_timeout_selector: B1,
    abort_task_set_timeout_selector: B1,
    clear_aca_timeout_selector: B1,
    clear_task_set_timeout_selector: B1,
    logical_unit_reset_timeout_selector: B1,
    query_task_timeout_selector: B1,
    reserved_4: B2,
    reserved_5: B5,
    query_asynchronous_event_timeout_selector: B1,
    query_task_set_timeout_selector: B1,
    i_t_nexus_reset_timeout_selector: B1,
    task_management_functions_long_timeout: B32,
    task_management_functions_short_timeout: B32,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = ReportSupportedTaskManagementFunctionsExtendedParameterData;

    type DataBufferWrapper = ReportSupportedTaskManagementFunctionsExtendedParameterData;

    type ReturnType = crate::Result<CommandResult>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        ReportSupportedTaskManagementFunctionsExtendedParameterData::new()
    }

    fn data_size(&self) -> u32 {
        self.command_buffer.allocation_length()
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let data = result.data;
        Ok(CommandResult {
            abort_task_supported: data.abort_task_supported() != 0,
            abort_task_set_supported: data.abort_task_set_supported() != 0,
            clear_aca_supported: data.clear_aca_supported() != 0,
            clear_task_set_supported: data.clear_task_set_supported() != 0,
            logical_unit_reset_supported: data.logical_unit_reset_supported() != 0,
            query_task_supported: data.query_task_supported() != 0,
            query_asynchronous_event_supported: data.query_asynchronous_event_supported() != 0,
            query_task_set_supported: data.query_task_set_supported() != 0,
            i_t_nexus_reset_supported: data.i_t_nexus_reset_supported() != 0,
            task_management_function_timeouts_valid: data.task_management_function_timeouts_valid()
                != 0,
            abort_task_timeout_selector: data.abort_task_timeout_selector() != 0,
            abort_task_set_timeout_selector: data.abort_task_set_timeout_selector() != 0,
            clear_aca_timeout_selector: data.clear_aca_timeout_selector() != 0,
            clear_task_set_timeout_selector: data.clear_task_set_timeout_selector() != 0,
            logical_unit_reset_timeout_selector: data.logical_unit_reset_timeout_selector() != 0,
            query_task_timeout_selector: data.query_task_timeout_selector() != 0,
            query_asynchronous_event_timeout_selector: data
                .query_asynchronous_event_timeout_selector()
                != 0,
            query_task_set_timeout_selector: data.query_task_set_timeout_selector() != 0,
            i_t_nexus_reset_timeout_selector: data.i_t_nexus_reset_timeout_selector() != 0,
            task_management_functions_long_timeout: data.task_management_functions_long_timeout(),
            task_management_functions_short_timeout: data.task_management_functions_short_timeout(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 12;
    const PARAMETER_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<ReportSupportedTaskManagementFunctionsExtendedParameterData>(),
            PARAMETER_LENGTH,
            concat!(
                "Size of: ",
                stringify!(ReportSupportedTaskManagementFunctionsExtendedParameterData)
            )
        );
    }
}
