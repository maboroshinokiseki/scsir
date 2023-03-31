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
pub struct PersistentReserveInCommand<'a> {
    interface: &'a Scsi,
    service_action: ServiceAction,
    command_buffer: CommandBuffer,
}

#[derive(Clone, Copy, Debug)]
pub enum ServiceAction {
    ReadKeys,
    ReadReservation,
    ReportCapabilities,
    ReadFullStatus,
    Other(u8),
}

pub enum CommandResult {
    ReadKeys(ReadKeysData),
    ReadReservation(ReadReservationData),
    ReportCapabilities(ReportCapabilitiesData),
    ReadFullStatus(ReadFullStatusData),
    Raw(Vec<u8>),
}

pub struct ReadKeysData {
    pub persistent_reservations_generation: u32,
    pub required_length: u32,
    pub reservation_keys: Vec<u64>,
}

pub struct ReadReservationData {
    pub persistent_reservations_generation: u32,
    pub reservation_key: u64,
    pub reservation_scope: u8,
    pub reservation_type: u8,
}

pub struct ReportCapabilitiesData {
    pub replace_lost_reservation_capable: bool,
    pub compatible_reservation_handling: bool,
    pub specify_initiator_ports_capable: bool,
    pub target_ports_capable: bool,
    pub persist_through_power_loss_capable: bool,
    pub type_mask_valid: bool,
    pub allow_commands: u8,
    pub persist_through_power_loss_activated: bool,
    pub write_exclusive_all_registrants: bool,
    pub exclusive_access_registrants_only: bool,
    pub write_exclusive_registrants_only: bool,
    pub exclusive_access: bool,
    pub write_exclusive: bool,
    pub exclusive_access_all_registrants: bool,
}

pub struct ReadFullStatusData {
    pub persistent_reservations_generation: u32,
    pub required_length: u32,
    pub descriptors: Vec<ReadFullStatusDescriptor>,
}

pub struct ReadFullStatusDescriptor {
    pub reservation_key: u64,
    pub all_target_ports: bool,
    pub reservation_holder: bool,
    pub reservation_scope: u8,
    pub reservation_type: u8,
    pub relative_target_port_identifier: u16,
    pub transportid: Vec<u8>,
}

impl<'a> PersistentReserveInCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            service_action: ServiceAction::ReadKeys,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
        }
    }

    // service_action must be less than 0x20
    pub fn service_action(&mut self, value: ServiceAction) -> &mut Self {
        self.service_action = value;
        self
    }

    pub fn allocation_length(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_allocation_length(value);
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<CommandResult> {
        bitfield_bound_check!(u8::from(self.service_action), 5, "service action")?;
        self.command_buffer
            .set_service_action(self.service_action.into());

        let temp = ThisCommand {
            command_buffer: self.command_buffer,
            service_action: self.service_action,
        };
        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn persistent_reserve_in(&self) -> PersistentReserveInCommand {
        PersistentReserveInCommand::new(self)
    }
}

impl ReadKeysData {
    fn from_bytes(bytes: &[u8]) -> Self {
        let (array, bytes) = get_array(bytes);
        let persistent_reservations_generation = u32::from_be_bytes(array);

        let (array, bytes) = get_array(bytes);
        let additional_length = u32::from_be_bytes(array);
        let required_length = additional_length.saturating_add(8);

        let mut reservation_keys = vec![];

        for chunk in bytes.chunks(size_of::<u64>()) {
            reservation_keys.push(u64::from_be_bytes(get_array(chunk).0))
        }

        Self {
            persistent_reservations_generation,
            required_length,
            reservation_keys,
        }
    }
}

impl ReadReservationData {
    fn from_bytes(bytes: &[u8]) -> Self {
        let (array, _) = get_array(bytes);
        let data = ReadReservationBitfield::from_bytes(array);

        Self {
            persistent_reservations_generation: data.persistent_reservations_generation(),
            reservation_key: data.reservation_key(),
            reservation_scope: data.reservation_scope(),
            reservation_type: data.reservation_type(),
        }
    }
}

impl ReportCapabilitiesData {
    fn from_bytes(bytes: &[u8]) -> Self {
        let (array, _) = get_array(bytes);
        let data = ReportCapabilitiesBitfield::from_bytes(array);

        Self {
            replace_lost_reservation_capable: data.replace_lost_reservation_capable() != 0,
            compatible_reservation_handling: data.compatible_reservation_handling() != 0,
            specify_initiator_ports_capable: data.specify_initiator_ports_capable() != 0,
            target_ports_capable: data.target_ports_capable() != 0,
            persist_through_power_loss_capable: data.persist_through_power_loss_capable() != 0,
            type_mask_valid: data.type_mask_valid() != 0,
            allow_commands: data.allow_commands(),
            persist_through_power_loss_activated: data.persist_through_power_loss_activated() != 0,
            write_exclusive_all_registrants: data.write_exclusive_all_registrants() != 0,
            exclusive_access_registrants_only: data.exclusive_access_registrants_only() != 0,
            write_exclusive_registrants_only: data.write_exclusive_registrants_only() != 0,
            exclusive_access: data.exclusive_access() != 0,
            write_exclusive: data.write_exclusive() != 0,
            exclusive_access_all_registrants: data.exclusive_access_all_registrants() != 0,
        }
    }
}

impl ReadFullStatusData {
    fn from_bytes(bytes: &[u8]) -> Self {
        let (array, mut bytes) = get_array(bytes);
        let header = ReadFullsstatusHeaderBitfield::from_bytes(array);

        let mut descriptors = vec![];

        while !bytes.is_empty() {
            let (array, left_bytes) = get_array(bytes);
            let descriptor_header = ReadFullsstatusDescriptorHeaderBitfield::from_bytes(array);
            let additional_descriptor_length = usize::min(
                descriptor_header.additional_descriptor_length() as usize,
                left_bytes.len(),
            );
            let transportid = Vec::from(&left_bytes[..additional_descriptor_length]);

            let descriptor = ReadFullStatusDescriptor {
                reservation_key: descriptor_header.reservation_key(),
                all_target_ports: descriptor_header.all_target_ports() != 0,
                reservation_holder: descriptor_header.reservation_holder() != 0,
                reservation_scope: descriptor_header.reservation_scope(),
                reservation_type: descriptor_header.reservation_type(),
                relative_target_port_identifier: descriptor_header
                    .relative_target_port_identifier(),
                transportid,
            };

            descriptors.push(descriptor);
            bytes = &left_bytes[additional_descriptor_length..];
        }

        Self {
            persistent_reservations_generation: header.persistent_reservations_generation(),
            required_length: header.additional_length().saturating_add(8),
            descriptors,
        }
    }
}

impl From<ServiceAction> for u8 {
    fn from(value: ServiceAction) -> Self {
        match value {
            ServiceAction::ReadKeys => 0x00,
            ServiceAction::ReadReservation => 0x01,
            ServiceAction::ReportCapabilities => 0x02,
            ServiceAction::ReadFullStatus => 0x03,
            ServiceAction::Other(x) => x,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReadReservationBitfield {
    persistent_reservations_generation: B32,
    additional_length: B32,
    reservation_key: B64,
    obsolete_0: B32,
    reserved: B8,
    reservation_scope: B4,
    reservation_type: B4,
    obsolete_1: B16,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReportCapabilitiesBitfield {
    length: B16,
    replace_lost_reservation_capable: B1,
    reserved_0: B2,
    compatible_reservation_handling: B1,
    specify_initiator_ports_capable: B1,
    target_ports_capable: B1,
    reserved_1: B1,
    persist_through_power_loss_capable: B1,
    type_mask_valid: B1,
    allow_commands: B3,
    reserved_2: B3,
    persist_through_power_loss_activated: B1,
    write_exclusive_all_registrants: B1,
    exclusive_access_registrants_only: B1,
    write_exclusive_registrants_only: B1,
    reserved_3: B1,
    exclusive_access: B1,
    reserved_4: B1,
    write_exclusive: B1,
    reserved_5: B1,
    reserved_6: B7,
    exclusive_access_all_registrants: B1,
    reserved_7: B16,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReadFullsstatusHeaderBitfield {
    persistent_reservations_generation: B32,
    additional_length: B32,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ReadFullsstatusDescriptorHeaderBitfield {
    reservation_key: B64,
    reserved_0: B32,
    reserved_1: B6,
    all_target_ports: B1,
    reservation_holder: B1,
    reservation_scope: B4,
    reservation_type: B4,
    reserved_2: B32,
    relative_target_port_identifier: B16,
    additional_descriptor_length: B32,
}

const OPERATION_CODE: u8 = 0x5E;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    reserved_1: B40,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    service_action: ServiceAction,
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
        self.command_buffer.allocation_length() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let bytes = &result.data()[..];

        Ok(match self.service_action {
            ServiceAction::ReadKeys => CommandResult::ReadKeys(ReadKeysData::from_bytes(bytes)),
            ServiceAction::ReadReservation => {
                CommandResult::ReadReservation(ReadReservationData::from_bytes(bytes))
            }
            ServiceAction::ReportCapabilities => {
                CommandResult::ReportCapabilities(ReportCapabilitiesData::from_bytes(bytes))
            }
            ServiceAction::ReadFullStatus => {
                CommandResult::ReadFullStatus(ReadFullStatusData::from_bytes(bytes))
            }
            ServiceAction::Other(_) => CommandResult::Raw(Vec::from(bytes)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 10;
    const READ_RESERVATION_BITFIELD_LENGTH: usize = 24;
    const REPORT_CAPABILITIES_BITFIELD_LENGTH: usize = 8;
    const READ_FULLSSTATUS_HEADER_BITFIELD_LENGTH: usize = 8;
    const READ_FULLSSTATUS_DESCRIPTOR_HEADER_BITFIELD_LENGTH: usize = 24;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<ReadReservationBitfield>(),
            READ_RESERVATION_BITFIELD_LENGTH,
            concat!("Size of: ", stringify!(ReadReservationBitfield))
        );

        assert_eq!(
            size_of::<ReportCapabilitiesBitfield>(),
            REPORT_CAPABILITIES_BITFIELD_LENGTH,
            concat!("Size of: ", stringify!(ReportCapabilitiesBitfield))
        );

        assert_eq!(
            size_of::<ReadFullsstatusHeaderBitfield>(),
            READ_FULLSSTATUS_HEADER_BITFIELD_LENGTH,
            concat!("Size of: ", stringify!(ReadFullsstatusHeaderBitfield))
        );

        assert_eq!(
            size_of::<ReadFullsstatusDescriptorHeaderBitfield>(),
            READ_FULLSSTATUS_DESCRIPTOR_HEADER_BITFIELD_LENGTH,
            concat!(
                "Size of: ",
                stringify!(ReadFullsstatusDescriptorHeaderBitfield)
            )
        );
    }
}
