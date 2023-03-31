#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::{LogParameter, ParameterHeader};

pub const PROTOCOL_SPECIFIC_PORT_PAGE_CODE: u8 = 0x18;
pub const PROTOCOL_SPECIFIC_PORT_SUBPAGE_CODE: u8 = 0x00;

#[derive(Clone, Debug)]
pub struct ProtocolSpecificPortParameter {
    pub header: ParameterHeader,
    pub body: ProtocolSpecificPortBody,
    pub log_descriptors: Vec<LogDescriptor>,
}

#[derive(Clone, Debug)]
pub struct LogDescriptor {
    pub header: SasPhyLogDescriptorHeader,
    pub event_descriptors: Vec<PhyEventDescriptor>,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ProtocolSpecificPortBody {
    reserved_0: B4,
    pub protocol_identifier: B4,
    reserved_1: B8,
    pub generation_code: B8,
    pub number_of_phys: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct SasPhyLogDescriptorHeader {
    reserved_0: B8,
    pub phy_identifier: B8,
    reserved_1: B8,
    pub sas_phy_log_descriptor_length: B8,
    reserved_2: B1,
    pub attached_device_type: B3,
    pub attached_reason: B4,
    pub reason: B4,
    pub negotiated_logical_link_rate: B4,
    reserved_3: B4,
    pub attached_ssp_initiator_port: B1,
    pub attached_stp_initiator_port: B1,
    pub attached_smp_initiator_port: B1,
    reserved_4: B1,
    reserved_5: B4,
    pub attached_ssp_target_port: B1,
    pub attached_stp_target_port: B1,
    pub attached_smp_target_port: B1,
    reserved_6: B1,
    pub sas_address: B64,
    pub attached_sas_address: B64,
    pub attached_phy_identifier: B8,
    reserved_7: B56,
    pub invalid_dword_count: B32,
    pub running_disparity_error_count: B32,
    pub loss_of_dword_synchronization: B32,
    pub phy_reset_problem: B32,
    reserved_8: B16,
    pub phy_event_descriptor_length: B8,
    pub number_of_phy_event_descriptors: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct PhyEventDescriptor {
    reserved: B24,
    pub phy_event_source: B8,
    pub phy_event: B32,
    pub peak_value_detector_threshold: B32,
}

impl LogParameter for ProtocolSpecificPortParameter {
    fn new() -> Self {
        Self {
            header: ParameterHeader::new(),
            body: ProtocolSpecificPortBody::new(),
            log_descriptors: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let header = ParameterHeader::from_bytes(array);
        let (array, mut bytes) = get_array(bytes);
        let body = ProtocolSpecificPortBody::from_bytes(array);
        let mut log_descriptors = vec![];
        for _ in 0..body.number_of_phys() {
            if bytes.is_empty() {
                break;
            }

            let descriptor;
            (descriptor, bytes) = LogDescriptor::from_bytes(bytes);
            log_descriptors.push(descriptor);
        }

        (
            Self {
                header,
                body,
                log_descriptors,
            },
            bytes,
        )
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.into_bytes());
        bytes.extend_from_slice(&self.body.bytes);
        for item in &self.log_descriptors {
            bytes.append(&mut item.to_bytes());
        }

        bytes
    }
}

impl LogDescriptor {
    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut bytes) = get_array(bytes);
        let header = SasPhyLogDescriptorHeader::from_bytes(array);
        let mut array;
        let mut event_descriptors = vec![];
        for _ in 0..header.number_of_phy_event_descriptors() {
            if bytes.is_empty() {
                break;
            }

            (array, bytes) = get_array(bytes);
            event_descriptors.push(PhyEventDescriptor::from_bytes(array));
        }

        (
            Self {
                header,
                event_descriptors,
            },
            bytes,
        )
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.bytes);
        for item in &self.event_descriptors {
            bytes.extend_from_slice(&item.bytes);
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PROTOCOL_SPECIFIC_PORT_BODY_LENGTH: usize = 4;
    const SAS_PHY_LOG_DESCRIPTOR_HEADER_LENGTH: usize = 52;
    const PHY_EVENT_DESCRIPTOR_LENGTH: usize = 12;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<ProtocolSpecificPortBody>(),
            PROTOCOL_SPECIFIC_PORT_BODY_LENGTH,
            concat!("Size of: ", stringify!(ProtocolSpecificPortBody))
        );

        assert_eq!(
            size_of::<SasPhyLogDescriptorHeader>(),
            SAS_PHY_LOG_DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(SasPhyLogDescriptorHeader))
        );

        assert_eq!(
            size_of::<PhyEventDescriptor>(),
            PHY_EVENT_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(PhyEventDescriptor))
        );
    }
}
