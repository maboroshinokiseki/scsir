#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReadCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    control: u8,
    group_number: u8,
    read_protect: u8,
    disable_page_out: bool,
    force_unit_access: bool,
    rebuild_assist_recovery_control: bool,
    logical_block_address: u64,
    expected_initial_logical_block_reference_tag: u32,
    expected_logical_block_application_tag: u16,
    logical_block_application_tag_mask: u16,
    transfer_length: u32,
    transfer_length_specified: bool,
    dld_0: bool,
    dld_1: bool,
    dld_2: bool,
    logical_block_size: u32,
}

#[derive(Clone, Copy)]
enum ReadCheckMode {
    Read6,
    Standard,
    WithDld,
    WithExpectedTags,
}

impl<'a> ReadCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            control: 0,
            group_number: 0,
            read_protect: 0,
            disable_page_out: false,
            force_unit_access: false,
            rebuild_assist_recovery_control: false,
            logical_block_address: 0,
            expected_initial_logical_block_reference_tag: 0,
            expected_logical_block_application_tag: 0,
            logical_block_application_tag_mask: 0,
            transfer_length: 0,
            transfer_length_specified: false,
            dld_0: false,
            dld_1: false,
            dld_2: false,
            logical_block_size: 512,
        }
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    // group_number must be less than 0x40 for read(16) or less than 0x20 for others
    pub fn group_number(&mut self, value: u8) -> &mut Self {
        self.group_number = value;
        self
    }

    // read_protect must be less than 0x08
    pub fn read_protect(&mut self, value: u8) -> &mut Self {
        self.read_protect = value;
        self
    }

    pub fn disable_page_out(&mut self, value: bool) -> &mut Self {
        self.disable_page_out = value;
        self
    }

    pub fn force_unit_access(&mut self, value: bool) -> &mut Self {
        self.force_unit_access = value;
        self
    }

    pub fn rebuild_assist_recovery_control(&mut self, value: bool) -> &mut Self {
        self.rebuild_assist_recovery_control = value;
        self
    }

    pub fn logical_block_address(&mut self, value: u64) -> &mut Self {
        self.logical_block_address = value;
        self
    }

    pub fn expected_initial_logical_block_reference_tag(&mut self, value: u32) -> &mut Self {
        self.expected_initial_logical_block_reference_tag = value;
        self
    }

    pub fn expected_logical_block_application_tag(&mut self, value: u16) -> &mut Self {
        self.expected_logical_block_application_tag = value;
        self
    }

    pub fn logical_block_application_tag_mask(&mut self, value: u16) -> &mut Self {
        self.logical_block_application_tag_mask = value;
        self
    }

    /// For `READ (6)`, this follows the SBC definition directly: a value of `0`
    /// encodes a 256 logical block transfer. `issue_6()` requires this field to be
    /// set explicitly so the default value does not accidentally issue a 256-block
    /// read.
    pub fn transfer_length(&mut self, value: u32) -> &mut Self {
        self.transfer_length = value;
        self.transfer_length_specified = true;
        self
    }

    pub fn dld_0(&mut self, value: bool) -> &mut Self {
        self.dld_0 = value;
        self
    }

    pub fn dld_1(&mut self, value: bool) -> &mut Self {
        self.dld_1 = value;
        self
    }

    pub fn dld_2(&mut self, value: bool) -> &mut Self {
        self.dld_2 = value;
        self
    }

    pub fn logical_block_size(&mut self, value: u32) -> &mut Self {
        self.logical_block_size = value;
        self
    }

    fn common_check(
        &self,
        group_number_bits: u32,
        logical_block_address_bits: u32,
        transfer_length_bits: u32,
        mode: ReadCheckMode,
    ) -> crate::Result<()> {
        if matches!(mode, ReadCheckMode::Read6) {
            if !self.transfer_length_specified {
                return Err(crate::Error::BadArgument(
                    "READ (6) requires transfer length to be set explicitly.".to_owned(),
                ));
            }

            if self.read_protect != 0
                || self.disable_page_out
                || self.force_unit_access
                || self.rebuild_assist_recovery_control
                || self.group_number != 0
                || self.dld_0
                || self.dld_1
                || self.dld_2
                || self.expected_initial_logical_block_reference_tag != 0
                || self.expected_logical_block_application_tag != 0
                || self.logical_block_application_tag_mask != 0
            {
                return Err(crate::Error::BadArgument(
                    "READ (6) only supports logical block address, transfer length, logical block size, timeout, and control."
                        .to_owned(),
                ));
            }
        } else {
            bitfield_bound_check!(self.read_protect, 3, "read protect")?;
            bitfield_bound_check!(self.group_number, group_number_bits, "group number")?;
        }

        bitfield_bound_check!(
            self.logical_block_address,
            logical_block_address_bits,
            "logical block address"
        )?;
        bitfield_bound_check!(
            self.transfer_length,
            transfer_length_bits,
            "transfer length"
        )?;
        bitfield_bound_check!(
            (self.effective_transfer_length(mode) as u64)
                .saturating_mul(self.logical_block_size as u64),
            32,
            "total transfer bytes"
        )?;

        if !matches!(mode, ReadCheckMode::Read6 | ReadCheckMode::WithDld)
            && (self.dld_0 || self.dld_1 || self.dld_2)
        {
            return Err(crate::Error::BadArgument(
                "DLDs are not allowed here".to_owned(),
            ));
        }

        if !matches!(mode, ReadCheckMode::Read6 | ReadCheckMode::WithExpectedTags)
            && (self.expected_initial_logical_block_reference_tag != 0
                || self.expected_logical_block_application_tag != 0
                || self.logical_block_application_tag_mask != 0)
        {
            return Err(crate::Error::BadArgument(
                "expected tags and mask are not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    fn effective_transfer_length(&self, mode: ReadCheckMode) -> u32 {
        match mode {
            ReadCheckMode::Read6 if self.transfer_length == 0 => 256,
            _ => self.transfer_length,
        }
    }

    pub fn issue_6(&mut self) -> crate::Result<Vec<u8>> {
        self.common_check(0, 21, 8, ReadCheckMode::Read6)?;

        let allocation_blocks = self.effective_transfer_length(ReadCheckMode::Read6);

        let command_buffer = CommandBuffer6::new()
            .with_operation_code(OPERATION_CODE_6)
            .with_logical_block_address(self.logical_block_address as u32)
            .with_transfer_length(self.transfer_length as u8)
            .with_control(self.control);

        let allocation_length = self.logical_block_size.saturating_mul(allocation_blocks);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length,
            timeout: self.timeout,
        })
    }

    pub fn issue_10(&mut self) -> crate::Result<Vec<u8>> {
        self.common_check(5, 32, 16, ReadCheckMode::Standard)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_read_protect(self.read_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_rebuild_assist_recovery_control(self.rebuild_assist_recovery_control.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_group_number(self.group_number)
            .with_transfer_length(self.transfer_length as u16)
            .with_control(self.control);

        let allocation_length = self.logical_block_size.saturating_mul(self.transfer_length);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length,
            timeout: self.timeout,
        })
    }

    pub fn issue_12(&mut self) -> crate::Result<Vec<u8>> {
        self.common_check(5, 32, 32, ReadCheckMode::Standard)?;

        let command_buffer = CommandBuffer12::new()
            .with_operation_code(OPERATION_CODE_12)
            .with_read_protect(self.read_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_rebuild_assist_recovery_control(self.rebuild_assist_recovery_control.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_group_number(self.group_number)
            .with_transfer_length(self.transfer_length)
            .with_control(self.control);

        let allocation_length = self.logical_block_size.saturating_mul(self.transfer_length);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length,
            timeout: self.timeout,
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<Vec<u8>> {
        self.common_check(6, 64, 32, ReadCheckMode::WithDld)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_read_protect(self.read_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_rebuild_assist_recovery_control(self.rebuild_assist_recovery_control.into())
            .with_logical_block_address(self.logical_block_address)
            .with_group_number(self.group_number)
            .with_transfer_length(self.transfer_length)
            .with_dld_0(self.dld_0.into())
            .with_dld_1(self.dld_1.into())
            .with_dld_2(self.dld_2.into())
            .with_control(self.control);

        let allocation_length = self.logical_block_size.saturating_mul(self.transfer_length);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length,
            timeout: self.timeout,
        })
    }

    pub fn issue_32(&mut self) -> crate::Result<Vec<u8>> {
        self.common_check(5, 64, 32, ReadCheckMode::WithExpectedTags)?;

        let command_buffer = CommandBuffer32::new()
            .with_operation_code(OPERATION_CODE_32)
            .with_control(self.control)
            .with_group_number(self.group_number)
            .with_additional_cdb_length(0x18)
            .with_service_action(SERVICE_ACTION_32)
            .with_read_protect(self.read_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_rebuild_assist_recovery_control(self.rebuild_assist_recovery_control.into())
            .with_logical_block_address(self.logical_block_address)
            .with_expected_initial_logical_block_reference_tag(
                self.expected_initial_logical_block_reference_tag,
            )
            .with_expected_logical_block_application_tag(
                self.expected_logical_block_application_tag,
            )
            .with_logical_block_application_tag_mask(self.logical_block_application_tag_mask)
            .with_transfer_length(self.transfer_length);

        let allocation_length = self.logical_block_size.saturating_mul(self.transfer_length);

        self.interface.issue(&ThisCommand {
            command_buffer,
            allocation_length,
            timeout: self.timeout,
        })
    }
}

impl Scsi {
    pub fn read(&self) -> ReadCommand<'_> {
        ReadCommand::new(self)
    }
}

const OPERATION_CODE_6: u8 = 0x08;
const OPERATION_CODE_10: u8 = 0x28;
const OPERATION_CODE_12: u8 = 0xA8;
const OPERATION_CODE_16: u8 = 0x88;
const OPERATION_CODE_32: u8 = 0x7F;
const SERVICE_ACTION_32: u16 = 0x0009;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer6 {
    operation_code: B8,
    reserved: B3,
    logical_block_address: B21,
    transfer_length: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    read_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    rebuild_assist_recovery_control: B1,
    obsolete: B2,
    logical_block_address: B32,
    reserved: B3,
    group_number: B5,
    transfer_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer12 {
    operation_code: B8,
    read_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    rebuild_assist_recovery_control: B1,
    obsolete: B2,
    logical_block_address: B32,
    transfer_length: B32,
    reserved: B3,
    group_number: B5,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    read_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    rebuild_assist_recovery_control: B1,
    obsolete: B1,
    dld_2: B1,
    logical_block_address: B64,
    transfer_length: B32,
    dld_1: B1,
    dld_0: B1,
    group_number: B6,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer32 {
    operation_code: B8,
    control: B8,
    reserved_0: B32,
    reserved_1: B3,
    group_number: B5,
    additional_cdb_length: B8,
    service_action: B16,
    read_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    rebuild_assist_recovery_control: B1,
    obsolete: B1,
    reserved_2: B1,
    reserved_3: B8,
    logical_block_address: B64,
    expected_initial_logical_block_reference_tag: B32,
    expected_logical_block_application_tag: B16,
    logical_block_application_tag_mask: B16,
    transfer_length: B32,
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

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { VecBufferWrapper::with_len(self.allocation_length as usize) }
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

    const COMMAND_LENGTH_6: usize = 6;
    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_12: usize = 12;
    const COMMAND_LENGTH_16: usize = 16;
    const COMMAND_LENGTH_32: usize = 32;

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

        assert_eq!(
            size_of::<CommandBuffer12>(),
            COMMAND_LENGTH_12,
            concat!("Size of: ", stringify!(CommandBuffer12))
        );

        assert_eq!(
            size_of::<CommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );

        assert_eq!(
            size_of::<CommandBuffer32>(),
            COMMAND_LENGTH_32,
            concat!("Size of: ", stringify!(CommandBuffer32))
        );
    }
}
