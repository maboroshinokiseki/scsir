#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::{bitfield_bound_check, get_array},
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

const HEADER_LENGTH: usize = 64;
const DESCRIPTOR_LENGTH: usize = 64;

#[derive(Clone, Debug)]
pub struct ReportZonesCommand<'a> {
    interface: &'a Scsi,
    timeout: Option<std::time::Duration>,
    reporting_options: ReportingOptions,
    descriptor_length: u32,
    command_buffer: CommandBuffer,
}

#[derive(Debug)]
pub struct CommandResult {
    pub zone_list_length: usize,
    pub same: Same,
    pub maximum_lba: u64,
    pub descriptors: Vec<ZoneDescriptor>,
}

#[derive(Debug)]
pub enum Same {
    TypeAndLengthMayDiffer,
    TypeAndLengthMatchFirst,
    TypeMatchesFirstLastLengthMayDiffer,
    LengthMatchesFirstTypeMayDiffer,
    Reserved(u8),
}

#[derive(Debug)]
pub struct ZoneDescriptor {
    pub zone_type: ZoneType,
    pub zone_condition: ZoneCondition,
    pub non_seq: bool,
    pub reset: bool,
    pub zone_length: u64,
    pub zone_start_lba: u64,
    pub write_pointer_lba: u64,
}

#[derive(Debug)]
pub enum ZoneType {
    Reserved,
    Conventional,
    SequentialWriteRequired,
    SequentialWritePreferred,
    Other(u8),
}

#[derive(Debug)]
pub enum ZoneCondition {
    NotWritePointer,
    Empty,
    ImplicitlyOpened,
    ExplicitlyOpened,
    Closed,
    ReadOnly,
    Full,
    Offline,
    Reserved(u8),
}

#[derive(Clone, Copy, Debug)]
pub enum ReportingOptions {
    All,
    Empty,
    ImplicitlyOpened,
    ExplicitlyOpened,
    Closed,
    Full,
    ReadOnly,
    Offline,
    RwpRecommended,
    NonSequentialWriteResourcesActive,
    NotWritePointer,
    Other(u8),
}

impl<'a> ReportZonesCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            timeout: None,
            reporting_options: ReportingOptions::All,
            descriptor_length: 0,
            command_buffer: CommandBuffer::new()
                .with_operation_code(OPERATION_CODE)
                .with_service_action(SERVICE_ACTION),
        }
    }

    pub fn zone_start_lba(&mut self, value: u64) -> &mut Self {
        self.command_buffer.set_zone_start_lba(value);
        self
    }

    pub fn partial(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_partial(value.into());
        self
    }

    pub fn reporting_options(&mut self, value: ReportingOptions) -> &mut Self {
        self.reporting_options = value;
        self
    }

    pub fn descriptor_length(&mut self, value: u32) -> &mut Self {
        self.descriptor_length = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        let reporting_options = self.reporting_options.value();
        let max_descriptor_length =
            ((u32::MAX as usize).saturating_sub(HEADER_LENGTH)) / DESCRIPTOR_LENGTH;

        bitfield_bound_check!(reporting_options, 6, "reporting options")?;

        if self.descriptor_length as usize > max_descriptor_length {
            return Err(crate::Error::ArgumentOutOfBounds(format!(
                "descriptor length is out of bounds. The maximum possible value is {}, but {} was provided.",
                max_descriptor_length, self.descriptor_length
            )));
        }

        let allocation_length =
            HEADER_LENGTH as u32 + self.descriptor_length * DESCRIPTOR_LENGTH as u32;

        self.interface.issue(&ThisCommand {
            command_buffer: self
                .command_buffer
                .with_allocation_length(allocation_length)
                .with_reporting_options(reporting_options),
            allocation_length,
            timeout: self.timeout,
        })
    }
}

impl Scsi {
    pub fn report_zones(&self) -> ReportZonesCommand<'_> {
        ReportZonesCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x95;
const SERVICE_ACTION: u8 = 0x00;

impl ReportingOptions {
    fn value(self) -> u8 {
        match self {
            ReportingOptions::All => 0x00,
            ReportingOptions::Empty => 0x01,
            ReportingOptions::ImplicitlyOpened => 0x02,
            ReportingOptions::ExplicitlyOpened => 0x03,
            ReportingOptions::Closed => 0x04,
            ReportingOptions::Full => 0x05,
            ReportingOptions::ReadOnly => 0x06,
            ReportingOptions::Offline => 0x07,
            ReportingOptions::RwpRecommended => 0x10,
            ReportingOptions::NonSequentialWriteResourcesActive => 0x11,
            ReportingOptions::NotWritePointer => 0x3F,
            ReportingOptions::Other(value) => value,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    zone_start_lba: B64,
    allocation_length: B32,
    partial: B1,
    reserved_1: B1,
    reporting_options: B6,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct ReportZonesParameterDataHeader {
    zone_list_length: B32,
    reserved_0: B4,
    same: B4,
    reserved_1: B24,
    maximum_lba: B64,
    reserved_2: B64,
    reserved_3: B64,
    reserved_4: B64,
    reserved_5: B64,
    reserved_6: B64,
    reserved_7: B64,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct RawDescriptor {
    reserved_0: B4,
    zone_type: B4,
    zone_condition: B4,
    reserved_1: B2,
    non_seq: B1,
    reset: B1,
    reserved_2: B48,
    zone_length: B64,
    zone_start_lba: B64,
    write_pointer_lba: B64,
    reserved_3: B64,
    reserved_4: B64,
    reserved_5: B64,
    reserved_6: B64,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    allocation_length: u32,
    timeout: Option<std::time::Duration>,
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

        let bytes = &result.data()[..result.transfered_data_length()];
        if bytes.len() < HEADER_LENGTH {
            return Err(crate::Error::Other(format!(
                "REPORT ZONES response is too short: {} bytes",
                bytes.len()
            )));
        }

        let (header_bytes, descriptor_bytes) = get_array(bytes);
        let raw_header = ReportZonesParameterDataHeader::from_bytes(header_bytes);

        let zone_list_length = raw_header.zone_list_length() as usize;
        let same = match raw_header.same() {
            0 => Same::TypeAndLengthMayDiffer,
            1 => Same::TypeAndLengthMatchFirst,
            2 => Same::TypeMatchesFirstLastLengthMayDiffer,
            3 => Same::LengthMatchesFirstTypeMayDiffer,
            other => Same::Reserved(other),
        };

        let maximum_lba = raw_header.maximum_lba();

        let descriptor_count =
            usize::min(zone_list_length, descriptor_bytes.len()) / DESCRIPTOR_LENGTH;

        let mut descriptors = Vec::with_capacity(descriptor_count);
        for raw in
            descriptor_bytes[..descriptor_count * DESCRIPTOR_LENGTH].chunks_exact(DESCRIPTOR_LENGTH)
        {
            let (raw_bytes, _) = get_array(raw);
            let raw_descriptor = RawDescriptor::from_bytes(raw_bytes);

            let zone_type = match raw_descriptor.zone_type() {
                0 => ZoneType::Reserved,
                1 => ZoneType::Conventional,
                2 => ZoneType::SequentialWriteRequired,
                3 => ZoneType::SequentialWritePreferred,
                other => ZoneType::Other(other),
            };

            let zone_condition = match raw_descriptor.zone_condition() {
                0 => ZoneCondition::NotWritePointer,
                1 => ZoneCondition::Empty,
                2 => ZoneCondition::ImplicitlyOpened,
                3 => ZoneCondition::ExplicitlyOpened,
                4 => ZoneCondition::Closed,
                0x0D => ZoneCondition::ReadOnly,
                0x0E => ZoneCondition::Full,
                0x0F => ZoneCondition::Offline,
                other => ZoneCondition::Reserved(other),
            };

            descriptors.push(ZoneDescriptor {
                zone_type,
                zone_condition,
                non_seq: raw_descriptor.non_seq() != 0,
                reset: raw_descriptor.reset() != 0,
                zone_length: raw_descriptor.zone_length(),
                zone_start_lba: raw_descriptor.zone_start_lba(),
                write_pointer_lba: raw_descriptor.write_pointer_lba(),
            });
        }

        Ok(CommandResult {
            zone_list_length,
            same,
            maximum_lba,
            descriptors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
        assert_eq!(
            size_of::<ReportZonesParameterDataHeader>(),
            HEADER_LENGTH,
            concat!("Size of: ", stringify!(ReportZonesParameterDataHeader))
        );
        assert_eq!(
            size_of::<RawDescriptor>(),
            DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(RawDescriptor))
        );
    }
}
