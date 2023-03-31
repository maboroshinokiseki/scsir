#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::{get_array, inquiry::InquiryCommand},
    data_wrapper::FlexibleStruct,
};

#[derive(Debug)]
pub struct DeviceIdentification {
    pub descriptors: Vec<IdentificationDescriptor>,
}

#[derive(Debug)]
pub struct IdentificationDescriptor {
    pub protocol_identifier: ProtocolIdentifier,
    pub association: Association,
    pub identifier_type: u8,
    pub identifier: Identifier,
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
pub enum Identifier {
    Binary(Vec<u8>),
    Ascii(String),
    Unknown(Vec<u8>),
}

#[derive(Debug)]
pub enum Association {
    AddressedPhysicalOrLogicalDevice,
    PortThatReceivedTheRequest,
    ScsiTargetDeviceThatContainsTheAddressedLogicalUnit,
    Other(u8),
}

pub fn device_identification(this: &mut InquiryCommand) -> crate::Result<DeviceIdentification> {
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
        (descriptor, bytes) = IdentificationDescriptor::from_bytes(bytes);
        descriptors.push(descriptor);
    }

    Ok(DeviceIdentification { descriptors })
}

impl IdentificationDescriptor {
    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, bytes) = get_array(bytes);
        let descriptor_header = DescriptorHeader::from_bytes(array);

        let protocol_identifier = if descriptor_header.protocol_identifier_valid() == 0
            || (descriptor_header.association() != 1 && descriptor_header.association() != 2)
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

        let association = match descriptor_header.association() {
            0x0 => Association::AddressedPhysicalOrLogicalDevice,
            0x1 => Association::PortThatReceivedTheRequest,
            0x2 => Association::ScsiTargetDeviceThatContainsTheAddressedLogicalUnit,
            other => Association::Other(other),
        };

        let (identifier_bytes, bytes) = bytes.split_at(usize::min(
            descriptor_header.identifier_length() as usize,
            bytes.len(),
        ));

        let identifier = match descriptor_header.code_set() {
            0x1 => Identifier::Binary(identifier_bytes.to_owned()),
            0x2 => Identifier::Ascii(String::from_utf8_lossy(identifier_bytes).to_string()),
            _ => Identifier::Unknown(identifier_bytes.to_owned()),
        };

        (
            IdentificationDescriptor {
                protocol_identifier,
                association,
                identifier_type: descriptor_header.identifier_type(),
                identifier,
            },
            bytes,
        )
    }
}

const PAGE_CODE: u8 = 0x83;

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
struct DescriptorHeader {
    protocol_identifier: B4,
    code_set: B4,
    protocol_identifier_valid: B1,
    reserved_0: B1,
    association: B2,
    identifier_type: B4,
    reserved_1: B8,
    identifier_length: B8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;
    const DESCRIPTOR_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );

        assert_eq!(
            size_of::<DescriptorHeader>(),
            DESCRIPTOR_HEADER_LENGTH,
            concat!("Size of: ", stringify!(DescriptorHeader))
        );
    }
}
