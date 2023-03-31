#![allow(dead_code)]

use std::vec;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::{get_array, inquiry::InquiryCommand},
    data_wrapper::FlexibleStruct,
};

#[derive(Debug)]
pub struct ScsiPorts {
    pub scsi_port_designation_descriptors: Vec<ScsiPortDesignationDescriptor>,
}

#[derive(Debug)]
pub struct ScsiPortDesignationDescriptor {
    pub relative_port_identifier: u16,
    pub initiator_port_transportid: Vec<u8>,
    pub target_port_descriptors: Vec<TargetPortDescriptor>,
}

#[derive(Debug)]
pub struct TargetPortDescriptor {
    pub protocol_identifier: ProtocolIdentifier,
    pub designator_type: u8,
    pub designator: Designator,
}

#[derive(Debug)]
pub enum ProtocolIdentifier {
    None,
    FibreChannel,
    Ssa,
    IEEE1394,
    RemoteDirectMemoryAccess,
    InternetScsi,
    SasSerialScsiProtocol,
    Other(u8),
}

#[derive(Debug)]
pub enum Designator {
    Binary(Vec<u8>),
    Ascii(String),
    Unknown(Vec<u8>),
}

pub fn scsi_ports(this: &mut InquiryCommand) -> crate::Result<ScsiPorts> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, u8> = this.issue_flex(0)?;
    let remaining = result.get_body().page_length();
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining as usize)?
    };

    let mut bytes = unsafe { result.elements_as_slice() };

    let mut descriptors = vec![];
    while !bytes.is_empty() {
        let descriptor;
        (descriptor, bytes) = ScsiPortDesignationDescriptor::from_bytes(bytes);

        descriptors.push(descriptor);
    }

    Ok(ScsiPorts {
        scsi_port_designation_descriptors: descriptors,
    })
}

impl ScsiPortDesignationDescriptor {
    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let descriptor_header = ScsiPortDesignationDescriptorHeader::from_bytes(array);
        let initiator_port_transportid_length =
            descriptor_header.initiator_port_transportid_length() as usize;
        let (initiator_port_transportid_bytes, bytes) =
            bytes.split_at(usize::min(initiator_port_transportid_length, bytes.len()));
        let initiator_port_transportid = initiator_port_transportid_bytes.to_vec();
        let (_, bytes) = get_array::<2>(bytes);
        let (array, bytes) = get_array(bytes);
        let target_port_descriptors_length = u16::from_be_bytes(array) as usize;
        let (mut target_port_descriptors_bytes, bytes) =
            bytes.split_at(usize::min(target_port_descriptors_length, bytes.len()));
        let mut target_port_descriptors = vec![];
        while !target_port_descriptors_bytes.is_empty() {
            let descriptor;
            (descriptor, target_port_descriptors_bytes) =
                TargetPortDescriptor::from_bytes(target_port_descriptors_bytes);
            target_port_descriptors.push(descriptor);
        }

        (
            Self {
                relative_port_identifier: descriptor_header.relative_port_identifier(),
                initiator_port_transportid,
                target_port_descriptors,
            },
            bytes,
        )
    }
}

impl TargetPortDescriptor {
    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let descriptor_header = TargetPortDescriptorHeader::from_bytes(array);

        let protocol_identifier = if descriptor_header.protocol_identifier_valid() != 1
            || (descriptor_header.association() != 1)
        {
            ProtocolIdentifier::None
        } else {
            match descriptor_header.protocol_identifier() {
                0x0 => ProtocolIdentifier::FibreChannel,
                0x2 => ProtocolIdentifier::Ssa,
                0x3 => ProtocolIdentifier::IEEE1394,
                0x4 => ProtocolIdentifier::RemoteDirectMemoryAccess,
                0x5 => ProtocolIdentifier::InternetScsi,
                0x6 => ProtocolIdentifier::SasSerialScsiProtocol,
                other => ProtocolIdentifier::Other(other),
            }
        };

        let (designator_bytes, bytes) = bytes.split_at(usize::min(
            descriptor_header.designator_length() as usize,
            bytes.len(),
        ));

        let designator = match descriptor_header.code_set() {
            0x1 => Designator::Binary(designator_bytes.to_owned()),
            0x2 => Designator::Ascii(String::from_utf8_lossy(designator_bytes).to_string()),
            _ => Designator::Unknown(designator_bytes.to_owned()),
        };

        (
            TargetPortDescriptor {
                protocol_identifier,
                designator_type: descriptor_header.designator_type(),
                designator,
            },
            bytes,
        )
    }
}

const PAGE_CODE: u8 = 0x88;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct PageHeader {
    peripheral_qualifier: B3,
    peripheral_device_type: B5,
    page_code: B8,
    page_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct ScsiPortDesignationDescriptorHeader {
    reserved_0: B16,
    relative_port_identifier: B16,
    reserved_1: B16,
    initiator_port_transportid_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct TargetPortDescriptorHeader {
    protocol_identifier: B4,
    code_set: B4,
    protocol_identifier_valid: B1,
    reserved_0: B1,
    association: B2,
    designator_type: B4,
    reserved_1: B8,
    designator_length: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;
    const SCSI_PORT_DESIGNATION_DESCRIPTOR_HEADER_LENGTH: usize = 8;
    const TARGET_PORT_DESCRIPTOR_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );

        assert_eq!(
            size_of::<ScsiPortDesignationDescriptorHeader>(),
            SCSI_PORT_DESIGNATION_DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ScsiPortDesignationDescriptorHeader))
        );

        assert_eq!(
            size_of::<TargetPortDescriptorHeader>(),
            TARGET_PORT_DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(TargetPortDescriptorHeader))
        );
    }
}
