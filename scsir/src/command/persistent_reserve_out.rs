#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct PersistentReserveOutCommand<'a> {
    interface: &'a Scsi,
    service_action: ServiceAction,
    reservation_scope: u8,
    reservation_type: u8,
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum ServiceAction {
    Register,
    Reserve,
    Release,
    Clear,
    Preempt,
    PreemptAndAbort,
    RegisterAndIgnoreExistingKey,
    RegisterAndMove,
    ReplaceLostReservation,
    Other(u8),
}

pub struct ParameterBuilder<'a> {
    parent: &'a mut PersistentReserveOutCommand<'a>,
    data_buffer: Vec<u8>,
}

pub struct BasicParameterData<'a> {
    parent: &'a mut ParameterBuilder<'a>,
    header: BasicParameterHeader,
    transport_id: Vec<u8>,
}

pub struct RegisterAndMoveParameterData<'a> {
    parent: &'a mut ParameterBuilder<'a>,
    header: RegisterAndMoveParameterHeader,
    transport_id: Vec<u8>,
}

impl<'a> PersistentReserveOutCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            service_action: ServiceAction::Register,
            reservation_scope: 0,
            reservation_type: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            data_buffer: vec![],
        }
    }

    pub fn service_action(&mut self, value: ServiceAction) -> &mut Self {
        self.service_action = value;
        self
    }

    pub fn reservation_scope(&mut self, value: u8) -> &mut Self {
        self.reservation_scope = value;
        self
    }

    pub fn reservation_type(&mut self, value: u8) -> &mut Self {
        self.reservation_type = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&'a mut self) -> ParameterBuilder<'a> {
        ParameterBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(u8::from(self.service_action), 5, "service action")?;
        bitfield_bound_check!(self.reservation_scope, 4, "reservation scope")?;
        bitfield_bound_check!(self.reservation_type, 4, "reservation type")?;
        bitfield_bound_check!(self.data_buffer.len(), 32, "parameter list length")?;

        let temp = ThisCommand {
            command_buffer: self
                .command_buffer
                .with_service_action(self.service_action.into())
                .with_reservation_scope(self.reservation_scope)
                .with_reservation_type(self.reservation_type)
                .with_parameter_list_length(self.data_buffer.len() as u32),
            data_buffer: self.data_buffer.clone().into(),
        };

        self.interface.issue(&temp)
    }
}

impl<'a> ParameterBuilder<'a> {
    fn new(parent: &'a mut PersistentReserveOutCommand<'a>) -> Self {
        Self {
            parent,
            data_buffer: vec![],
        }
    }

    pub fn basic_parameter(&'a mut self) -> BasicParameterData<'a> {
        BasicParameterData::new(self)
    }

    pub fn register_and_move_parameter(&'a mut self) -> RegisterAndMoveParameterData<'a> {
        RegisterAndMoveParameterData::new(self)
    }

    pub fn done(&'a mut self) -> &'a mut PersistentReserveOutCommand {
        self.parent.data_buffer = std::mem::take(&mut self.data_buffer);
        self.parent
    }
}

impl<'a> BasicParameterData<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            parent,
            header: BasicParameterHeader::new(),
            transport_id: vec![],
        }
    }

    pub fn reservation_key(&mut self, value: u64) -> &mut Self {
        self.header.set_reservation_key(value);
        self
    }

    pub fn service_action_reservation_key(&mut self, value: u64) -> &mut Self {
        self.header.set_service_action_reservation_key(value);
        self
    }

    pub fn specify_initiator_ports(&mut self, value: bool) -> &mut Self {
        self.header.set_specify_initiator_ports(value as u8);
        self
    }

    pub fn all_target_ports(&mut self, value: bool) -> &mut Self {
        self.header.set_all_target_ports(value as u8);
        self
    }

    pub fn activate_persist_through_power_loss(&mut self, value: bool) -> &mut Self {
        self.header
            .set_activate_persist_through_power_loss(value as u8);
        self
    }

    pub fn transport_id_list(&mut self, value: &[u8]) -> &mut Self {
        self.transport_id.clear();
        self.transport_id.extend_from_slice(value);
        self
    }

    pub fn done(&'a mut self) -> &'a mut ParameterBuilder {
        self.parent.data_buffer.clear();
        self.parent
            .data_buffer
            .extend_from_slice(&self.header.bytes);
        if self.header.specify_initiator_ports() != 0 {
            self.parent
                .data_buffer
                .extend_from_slice(&(self.transport_id.len() as u32).to_be_bytes());
            self.parent.data_buffer.append(&mut self.transport_id);
        }

        self.parent
    }
}

impl<'a> RegisterAndMoveParameterData<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            parent,
            header: RegisterAndMoveParameterHeader::new(),
            transport_id: vec![],
        }
    }

    pub fn reservation_key(&mut self, value: u64) -> &mut Self {
        self.header.set_reservation_key(value);
        self
    }

    pub fn service_action_reservation_key(&mut self, value: u64) -> &mut Self {
        self.header.set_service_action_reservation_key(value);
        self
    }

    pub fn unregister(&mut self, value: bool) -> &mut Self {
        self.header.set_unregister(value as u8);
        self
    }

    pub fn activate_persist_through_power_loss(&mut self, value: bool) -> &mut Self {
        self.header
            .set_activate_persist_through_power_loss(value as u8);
        self
    }

    pub fn relative_target_port_identifier(&mut self, value: u16) -> &mut Self {
        self.header.set_relative_target_port_identifier(value);
        self
    }

    pub fn transport_id_list(&mut self, value: &[u8]) -> &mut Self {
        self.transport_id.clear();
        self.transport_id.extend_from_slice(value);
        self
    }

    pub fn done(&'a mut self) -> &'a mut ParameterBuilder {
        self.parent.data_buffer.clear();
        self.header
            .set_transportid_parameter_data_length(self.transport_id.len() as u32);
        self.parent
            .data_buffer
            .extend_from_slice(&self.header.bytes);
        self.parent.data_buffer.append(&mut self.transport_id);

        self.parent
    }
}

impl Scsi {
    pub fn persistent_reserve_out(&self) -> PersistentReserveOutCommand {
        PersistentReserveOutCommand::new(self)
    }
}

impl From<ServiceAction> for u8 {
    fn from(value: ServiceAction) -> Self {
        match value {
            ServiceAction::Register => 0x00,
            ServiceAction::Reserve => 0x01,
            ServiceAction::Release => 0x02,
            ServiceAction::Clear => 0x03,
            ServiceAction::Preempt => 0x04,
            ServiceAction::PreemptAndAbort => 0x05,
            ServiceAction::RegisterAndIgnoreExistingKey => 0x06,
            ServiceAction::RegisterAndMove => 0x07,
            ServiceAction::ReplaceLostReservation => 0x08,
            ServiceAction::Other(x) => x,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy)]
struct BasicParameterHeader {
    reservation_key: B64,
    service_action_reservation_key: B64,
    obsolete_0: B32,
    reserved_0: B4,
    specify_initiator_ports: B1,
    all_target_ports: B1,
    reserved_1: B1,
    activate_persist_through_power_loss: B1,
    reserved_2: B8,
    obsolete_1: B16,
}

#[bitfield]
#[derive(Clone, Copy)]
struct RegisterAndMoveParameterHeader {
    reservation_key: B64,
    service_action_reservation_key: B64,
    reserved_0: B8,
    reserved_1: B6,
    unregister: B1,
    activate_persist_through_power_loss: B1,
    relative_target_port_identifier: B16,
    transportid_parameter_data_length: B32,
}

const OPERATION_CODE: u8 = 0x5F;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B3,
    service_action: B5,
    reservation_scope: B4,
    reservation_type: B4,
    reserved_1: B16,
    parameter_list_length: B32,
    control: B8,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: VecBufferWrapper,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = VecBufferWrapper;

    type ReturnType = crate::Result<()>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        self.data_buffer.clone()
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

    const COMMAND_LENGTH: usize = 10;
    const BASIC_PARAMETER_HEADER_LENGTH: usize = 24;
    const REGISTER_AND_MOVE_PARAMETER_HEADER_LENGTH: usize = 24;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<BasicParameterHeader>(),
            BASIC_PARAMETER_HEADER_LENGTH,
            concat!("Size of: ", stringify!(BasicParameterHeader))
        );

        assert_eq!(
            size_of::<RegisterAndMoveParameterHeader>(),
            REGISTER_AND_MOVE_PARAMETER_HEADER_LENGTH,
            concat!("Size of: ", stringify!(RegisterAndMoveParameterHeader))
        );
    }
}
