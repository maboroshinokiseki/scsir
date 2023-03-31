#![allow(dead_code)]

use std::mem::size_of;

use modular_bitfield_msb::prelude::*;

use crate::{command::inquiry::InquiryCommand, data_wrapper::FlexibleStruct};

#[derive(Debug)]
pub struct DeviceIdentification {
    pub descriptors: Vec<PowerConsumptionDescriptor>,
}

#[derive(Debug)]
pub struct PowerConsumptionDescriptor {
    pub power_consumption_identifier: u8,
    pub power_consumption_in_microwatts: u64,
}
pub fn power_consumption(this: &mut InquiryCommand) -> crate::Result<DeviceIdentification> {
    this.page_code(Some(PAGE_CODE));

    let result: FlexibleStruct<PageHeader, Descriptor> = this.issue_flex(0)?;
    let remaining = result.get_body().page_length() as usize / size_of::<Descriptor>();
    let result = if remaining == 0 {
        result
    } else {
        this.issue_flex(remaining)?
    };

    let mut descriptors = vec![];

    for item in unsafe { result.elements_as_slice() } {
        let multiplier = match item.power_consumption_units() {
            0b0101 => 1,
            0b0100 => 1_000,
            0b0011 => 1_000_000,
            0b0010 => 1_000_000_000,
            0b0001 => 1_000_000_000_000,
            0b0000 => 1_000_000_000_000_000,
            _ => 0,
        };

        let descriptor = PowerConsumptionDescriptor {
            power_consumption_identifier: item.power_consumption_identifier(),
            power_consumption_in_microwatts: item.power_consumption_value() as u64 * multiplier,
        };

        descriptors.push(descriptor);
    }

    Ok(DeviceIdentification { descriptors })
}

const PAGE_CODE: u8 = 0x8D;

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
struct Descriptor {
    power_consumption_identifier: B8,
    reserved: B5,
    power_consumption_units: B3,
    power_consumption_value: B16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 4;
    const DESCRIPTOR_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<PageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(PageHeader))
        );

        assert_eq!(
            size_of::<Descriptor>(),
            DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(Descriptor))
        );
    }
}
