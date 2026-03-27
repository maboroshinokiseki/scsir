#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct WriteCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    control: u8,
    group_number: u8,
    write_protect: u8,
    disable_page_out: bool,
    force_unit_access: bool,
    logical_block_address: u64,
    expected_initial_logical_block_reference_tag: u32,
    expected_logical_block_application_tag: u16,
    logical_block_application_tag_mask: u16,
    dld_0: bool,
    dld_1: bool,
    dld_2: bool,
    logical_block_size: u32,
    data_buffer: Vec<u8>,
}

enum WriteCheckMode {
    Write6,
    Standard,
    WithDld,
    WithExpectedTags,
}

impl<'a> WriteCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            control: 0,
            group_number: 0,
            write_protect: 0,
            disable_page_out: false,
            force_unit_access: false,
            logical_block_address: 0,
            expected_initial_logical_block_reference_tag: 0,
            expected_logical_block_application_tag: 0,
            logical_block_application_tag_mask: 0,
            dld_0: false,
            dld_1: false,
            dld_2: false,
            logical_block_size: 512,
            data_buffer: vec![],
        }
    }

    // group_number must be less than 0x40 for write(16) or less than 0x20 for others
    pub fn group_number(&mut self, value: u8) -> &mut Self {
        self.group_number = value;
        self
    }

    // write_protect must be less than 0x08
    pub fn write_protect(&mut self, value: u8) -> &mut Self {
        self.write_protect = value;
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

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
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

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    fn error_check(
        &self,
        group_number_bits: u32,
        logical_block_address_bits: u32,
        transfer_length_bits: u32,
        mode: WriteCheckMode,
    ) -> crate::Result<()> {
        let transfer_length = self.data_transfer_length();

        if matches!(mode, WriteCheckMode::Write6) {
            if self.write_protect != 0
                || self.disable_page_out
                || self.force_unit_access
                || self.group_number != 0
                || self.dld_0
                || self.dld_1
                || self.dld_2
                || self.expected_initial_logical_block_reference_tag != 0
                || self.expected_logical_block_application_tag != 0
                || self.logical_block_application_tag_mask != 0
            {
                return Err(crate::Error::BadArgument(
                    "WRITE (6) only supports logical block address, parameter data, logical block size, timeout, and control."
                        .to_owned(),
                ));
            }

            if transfer_length == 0 || transfer_length > 256 {
                return Err(crate::Error::ArgumentOutOfBounds(
                    "parameter length is out of bounds for WRITE (6). The transfer length must be between 1 and 256 logical blocks."
                        .to_owned(),
                ));
            }
        } else {
            bitfield_bound_check!(self.group_number, group_number_bits, "group number")?;
            bitfield_bound_check!(self.write_protect, 3, "verify protect")?;

            if transfer_length.wrapping_shr(transfer_length_bits) != 0 {
                return Err(crate::Error::ArgumentOutOfBounds(format!(
                    "parameter length is out of bounds. The maximum possible value is {}, but {} was provided.",
                    1u128.wrapping_shl(transfer_length_bits) * self.logical_block_size as u128,
                    self.data_buffer.len()
                )));
            }
        }

        bitfield_bound_check!(
            self.logical_block_address,
            logical_block_address_bits,
            "logical block address"
        )?;

        if self.data_buffer.len() % self.logical_block_size as usize != 0 {
            return Err(crate::Error::BadArgument(format!(
                "parameter length should be a multiple of logical block size, which is {}.",
                self.logical_block_size
            )));
        }

        if self.data_buffer.len() > u32::MAX as usize {
            return Err(crate::Error::ArgumentOutOfBounds(format!(
                "parameter length is out of bounds. The maximum transport byte count is {}, but {} was provided.",
                u32::MAX,
                self.data_buffer.len()
            )));
        }

        if !matches!(mode, WriteCheckMode::Write6 | WriteCheckMode::WithDld)
            && (self.dld_0 || self.dld_1 || self.dld_2)
        {
            return Err(crate::Error::BadArgument(
                "DLDs are not allowed here".to_owned(),
            ));
        }

        if !matches!(
            mode,
            WriteCheckMode::Write6 | WriteCheckMode::WithExpectedTags
        ) && (self.expected_initial_logical_block_reference_tag != 0
            || self.expected_logical_block_application_tag != 0
            || self.logical_block_application_tag_mask != 0)
        {
            return Err(crate::Error::BadArgument(
                "expected tags and mask are not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    fn data_transfer_length(&self) -> usize {
        self.data_buffer.len() / self.logical_block_size as usize
    }

    pub fn issue_6(&mut self) -> crate::Result<()> {
        self.error_check(0, 21, 8, WriteCheckMode::Write6)?;

        let transfer_length = self.data_transfer_length();
        let encoded_transfer_length = if transfer_length == 256 {
            0
        } else {
            transfer_length as u8
        };

        let command_buffer = CommandBuffer6::new()
            .with_operation_code(OPERATION_CODE_6)
            .with_logical_block_address(self.logical_block_address as u32)
            .with_transfer_length(encoded_transfer_length)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        })
    }

    pub fn issue_10(&mut self) -> crate::Result<()> {
        self.error_check(5, 32, 16, WriteCheckMode::Standard)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_group_number(self.group_number)
            .with_transfer_length(self.data_transfer_length() as u16)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        })
    }

    pub fn issue_12(&mut self) -> crate::Result<()> {
        self.error_check(5, 32, 32, WriteCheckMode::Standard)?;

        let command_buffer = CommandBuffer12::new()
            .with_operation_code(OPERATION_CODE_12)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address as u32)
            .with_transfer_length(self.data_transfer_length() as u32)
            .with_group_number(self.group_number)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<()> {
        self.error_check(6, 64, 32, WriteCheckMode::WithDld)?;

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address)
            .with_transfer_length(self.data_transfer_length() as u32)
            .with_dld_0(self.dld_0.into())
            .with_dld_1(self.dld_1.into())
            .with_dld_2(self.dld_2.into())
            .with_group_number(self.group_number)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        })
    }

    pub fn issue_32(&mut self) -> crate::Result<()> {
        self.error_check(5, 64, 32, WriteCheckMode::WithExpectedTags)?;

        let command_buffer = CommandBuffer32::new()
            .with_operation_code(OPERATION_CODE_32)
            .with_control(self.control)
            .with_group_number(self.group_number)
            .with_additional_cdb_length(0x18)
            .with_service_action(SERVICE_ACTION_32)
            .with_write_protect(self.write_protect)
            .with_disable_page_out(self.disable_page_out.into())
            .with_force_unit_access(self.force_unit_access.into())
            .with_logical_block_address(self.logical_block_address)
            .with_expected_initial_logical_block_reference_tag(
                self.expected_initial_logical_block_reference_tag,
            )
            .with_expected_logical_block_application_tag(
                self.expected_logical_block_application_tag,
            )
            .with_logical_block_application_tag_mask(self.logical_block_application_tag_mask)
            .with_transfer_length(self.data_transfer_length() as u32);

        self.interface.issue(&ThisCommand {
            command_buffer,
            data_buffer: self.data_buffer.clone().into(),
            timeout: self.timeout,
        })
    }
}

impl Scsi {
    pub fn write(&self) -> WriteCommand<'_> {
        WriteCommand::new(self)
    }
}

const OPERATION_CODE_6: u8 = 0x0A;
const OPERATION_CODE_10: u8 = 0x2A;
const OPERATION_CODE_12: u8 = 0xAA;
const OPERATION_CODE_16: u8 = 0x8A;
const OPERATION_CODE_32: u8 = 0x7F;
const SERVICE_ACTION_32: u16 = 0x000B;

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
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved_0: B1,
    obsolete: B2,
    logical_block_address: B32,
    reserved_1: B3,
    group_number: B5,
    transfer_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer12 {
    operation_code: B8,
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved_0: B1,
    obsolete: B2,
    logical_block_address: B32,
    transfer_length: B32,
    reserved_1: B3,
    group_number: B5,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved: B1,
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
    write_protect: B3,
    disable_page_out: B1,
    force_unit_access: B1,
    reserved_2: B1,
    obsolete: B1,
    reserved_3: B1,
    reserved_4: B8,
    logical_block_address: B64,
    expected_initial_logical_block_reference_tag: B32,
    expected_logical_block_application_tag: B16,
    logical_block_application_tag_mask: B16,
    transfer_length: B32,
}

struct ThisCommand<C> {
    command_buffer: C,
    data_buffer: VecBufferWrapper,
    timeout: Option<std::time::Duration>,
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

    fn timeout_override(&self) -> Option<std::time::Duration> {
        self.timeout
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
