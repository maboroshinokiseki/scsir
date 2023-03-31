#![allow(dead_code)]

use std::{marker::PhantomData, mem::size_of};

use modular_bitfield_msb::prelude::*;

use crate::{
    command::{bitfield_bound_check, get_array},
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct ReadDefectDataCommand<'a> {
    interface: &'a Scsi,
    request_primary_defect_list: bool,
    request_grown_defect_list: bool,
    defect_list_format: u8,
    address_descriptor_index: u32,
    descriptor_length: u32,
    control: u8,
}

#[derive(Clone, Debug)]
pub struct CommandResult {
    pub primary_defect_list_valid: bool,
    pub grown_defect_list_valid: bool,
    pub total_descriptor_length: u32,
    pub descriptors: DefectList,
}

#[derive(Clone, Debug)]
pub enum DefectList {
    ShortBlockFormat(Vec<ShortBlockFormatAddressDescriptor>),
    ExtendedBytesFromIndex(Vec<ExtendedBytesFromIndexAddressDescriptor>),
    ExtendedPhysicalSector(Vec<ExtendedPhysicalSectorAddressDescriptor>),
    LongBlockFormat(Vec<LongBlockFormatAddressDescriptor>),
    BytesFromIndexFormat(Vec<BytesFromIndexFormatAddressDescriptor>),
    PhysicalSectorFormat(Vec<PhysicalSectorFormatAddressDescriptor>),
    Custom(Vec<u8>),
}

#[derive(Clone, Copy, Debug)]
pub struct ShortBlockFormatAddressDescriptor {
    pub short_block_address: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ExtendedBytesFromIndexAddressDescriptor {
    pub cylinder_number: u32,
    pub head_number: u8,
    pub multi_address_descriptor_start: bool,
    pub bytes_from_index: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ExtendedPhysicalSectorAddressDescriptor {
    pub cylinder_number: u32,
    pub head_number: u8,
    pub multi_address_descriptor_start: bool,
    pub sector_number: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct LongBlockFormatAddressDescriptor {
    pub long_block_address: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct BytesFromIndexFormatAddressDescriptor {
    pub cylinder_number: u32,
    pub head_number: u8,
    pub bytes_from_index: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PhysicalSectorFormatAddressDescriptor {
    pub cylinder_number: u32,
    pub head_number: u8,
    pub sector_number: u32,
}

impl<'a> ReadDefectDataCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            request_primary_defect_list: false,
            request_grown_defect_list: false,
            defect_list_format: 0,
            address_descriptor_index: 0,
            descriptor_length: 0,
            control: 0,
        }
    }

    pub fn request_primary_defect_list(&mut self, value: bool) -> &mut Self {
        self.request_primary_defect_list = value;
        self
    }

    pub fn request_grown_defect_list(&mut self, value: bool) -> &mut Self {
        self.request_grown_defect_list = value;
        self
    }

    // defect_list_format must be less than 0x08
    pub fn defect_list_format(&mut self, value: u8) -> &mut Self {
        self.defect_list_format = value;
        self
    }

    pub fn address_descriptor_index(&mut self, value: u32) -> &mut Self {
        self.address_descriptor_index = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.control = value;
        self
    }

    pub fn descriptor_length(&mut self, value: u32) -> &mut Self {
        self.descriptor_length = value;
        self
    }

    fn error_check(
        &self,
        header_size: usize,
        max_allocation_length: usize,
        allow_address_descriptor_index: bool,
    ) -> crate::Result<()> {
        bitfield_bound_check!(self.defect_list_format, 3, "defect list format")?;

        let max_descriptor_length =
            (max_allocation_length - header_size) / self.get_defect_list_item_size();
        if self.descriptor_length > max_descriptor_length as u32 {
            return Err(
                crate::Error::ArgumentOutOfBounds(
                    format!(
                        "Expected descriptor length is out of bounds. The maximum possible value is {}, but {} was provided.",
                        max_descriptor_length,
                        self.descriptor_length)));
        }

        if !allow_address_descriptor_index && self.address_descriptor_index > 0 {
            return Err(crate::Error::BadArgument(
                "address descriptor index is not allowed here".to_owned(),
            ));
        }

        Ok(())
    }

    fn get_defect_list_item_size(&self) -> usize {
        match self.defect_list_format {
            0b0000 => size_of::<super::format_unit::ShortBlockFormatAddressDescriptor>(),
            0b0001 => size_of::<super::format_unit::ExtendedBytesFromIndexAddressDescriptor>(),
            0b0010 => size_of::<super::format_unit::ExtendedPhysicalSectorAddressDescriptor>(),
            0b0011 => size_of::<super::format_unit::LongBlockFormatAddressDescriptor>(),
            0b0100 => size_of::<super::format_unit::BytesFromIndexFormatAddressDescriptor>(),
            0b0101 => size_of::<super::format_unit::PhysicalSectorFormatAddressDescriptor>(),
            _ => size_of::<u8>(),
        }
    }

    pub fn issue_10(&mut self) -> crate::Result<CommandResult> {
        let extra_allocation_length =
            self.descriptor_length as usize * self.get_defect_list_item_size();
        let allocation_length = size_of::<DataBufferHeader10>() + extra_allocation_length;

        self.error_check(size_of::<DataBufferHeader10>(), u16::MAX.into(), false)?;

        let command_buffer = CommandBuffer10::new()
            .with_operation_code(OPERATION_CODE_10)
            .with_request_primary_defect_list(self.request_primary_defect_list.into())
            .with_request_grown_defect_list(self.request_grown_defect_list.into())
            .with_defect_list_format(self.defect_list_format)
            .with_allocation_length(allocation_length as u16)
            .with_control(self.control);

        let (body, defect_list) = self.interface.issue(&ThisCommand {
            command_buffer,
            extra_allocation_length,
            defect_list_format: self.defect_list_format,
            marker: PhantomData::<DataBufferHeader10>,
        })?;

        Ok(CommandResult {
            primary_defect_list_valid: body.primary_defect_list_valid() != 0,
            grown_defect_list_valid: body.grown_defect_list_valid() != 0,
            total_descriptor_length: (body.defect_list_length() as usize
                / self.get_defect_list_item_size()) as u32,
            descriptors: defect_list,
        })
    }

    pub fn issue_12(&mut self) -> crate::Result<CommandResult> {
        let extra_allocation_length =
            self.descriptor_length as usize * self.get_defect_list_item_size();
        let allocation_length = size_of::<DataBufferHeader12>() + extra_allocation_length;

        self.error_check(size_of::<DataBufferHeader12>(), u32::MAX as usize, true)?;

        let command_buffer = CommandBuffer12::new()
            .with_operation_code(OPERATION_CODE_12)
            .with_request_primary_defect_list(self.request_primary_defect_list.into())
            .with_request_grown_defect_list(self.request_grown_defect_list.into())
            .with_defect_list_format(self.defect_list_format)
            .with_address_descriptor_index(self.address_descriptor_index)
            .with_allocation_length(allocation_length as u32)
            .with_control(self.control);

        let (body, defect_list) = self.interface.issue(&ThisCommand {
            command_buffer,
            extra_allocation_length,
            defect_list_format: self.defect_list_format,
            marker: PhantomData::<DataBufferHeader12>,
        })?;

        Ok(CommandResult {
            primary_defect_list_valid: body.primary_defect_list_valid() != 0,
            grown_defect_list_valid: body.grown_defect_list_valid() != 0,
            total_descriptor_length: (body.defect_list_length() as usize
                / self.get_defect_list_item_size()) as u32,
            descriptors: defect_list,
        })
    }
}

impl Scsi {
    pub fn read_defect_data(&self) -> ReadDefectDataCommand {
        ReadDefectDataCommand::new(self)
    }
}

const OPERATION_CODE_10: u8 = 0x37;
const OPERATION_CODE_12: u8 = 0xB7;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer10 {
    operation_code: B8,
    reserved_0: B8,
    reserved_1: B3,
    request_primary_defect_list: B1,
    request_grown_defect_list: B1,
    defect_list_format: B3,
    reserved_2: B32,
    allocation_length: B16,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer12 {
    operation_code: B8,
    reserved_0: B3,
    request_primary_defect_list: B1,
    request_grown_defect_list: B1,
    defect_list_format: B3,
    address_descriptor_index: B32,
    allocation_length: B32,
    reserved_1: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct DataBufferHeader10 {
    reserved_0: B8,
    reserved_1: B3,
    primary_defect_list_valid: B1,
    grown_defect_list_valid: B1,
    defect_list_format: B3,
    defect_list_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct DataBufferHeader12 {
    reserved_0: B8,
    reserved_1: B3,
    primary_defect_list_valid: B1,
    grown_defect_list_valid: B1,
    defect_list_format: B3,
    reserved_2: B16,
    defect_list_length: B32,
}

struct ThisCommand<C, Body> {
    command_buffer: C,
    extra_allocation_length: usize,
    defect_list_format: u8,

    marker: PhantomData<Body>,
}

impl<C: Copy, Body: Copy> Command for ThisCommand<C, Body> {
    type CommandBuffer = C;

    type DataBuffer = AnyType;

    type DataBufferWrapper = FlexibleStruct<Body, u8>;

    type ReturnType = crate::Result<(Body, DefectList)>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { FlexibleStruct::with_length(self.extra_allocation_length) }
    }

    fn data_size(&self) -> u32 {
        (self.extra_allocation_length + size_of::<Body>()) as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        let mut defect_list = match self.defect_list_format {
            0b0000 => DefectList::ShortBlockFormat(vec![]),
            0b0001 => DefectList::ExtendedBytesFromIndex(vec![]),
            0b0010 => DefectList::ExtendedPhysicalSector(vec![]),
            0b0011 => DefectList::LongBlockFormat(vec![]),
            0b0100 => DefectList::BytesFromIndexFormat(vec![]),
            0b0101 => DefectList::PhysicalSectorFormat(vec![]),
            _ => DefectList::Custom(vec![]),
        };

        match &mut defect_list {
            DefectList::ShortBlockFormat(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::ShortBlockFormatAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw =
                        super::format_unit::ShortBlockFormatAddressDescriptor::from_bytes(bytes);
                    v.push(ShortBlockFormatAddressDescriptor {
                        short_block_address: raw.short_block_address(),
                    });
                }
            }
            DefectList::ExtendedBytesFromIndex(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::ExtendedBytesFromIndexAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw =
                        super::format_unit::ExtendedBytesFromIndexAddressDescriptor::from_bytes(
                            bytes,
                        );
                    v.push(ExtendedBytesFromIndexAddressDescriptor {
                        cylinder_number: raw.cylinder_number(),
                        head_number: raw.head_number(),
                        multi_address_descriptor_start: raw.multi_address_descriptor_start() != 0,
                        bytes_from_index: raw.bytes_from_index(),
                    });
                }
            }
            DefectList::ExtendedPhysicalSector(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::ExtendedPhysicalSectorAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw =
                        super::format_unit::ExtendedPhysicalSectorAddressDescriptor::from_bytes(
                            bytes,
                        );
                    v.push(ExtendedPhysicalSectorAddressDescriptor {
                        cylinder_number: raw.cylinder_number(),
                        head_number: raw.head_number(),
                        multi_address_descriptor_start: raw.multi_address_descriptor_start() != 0,
                        sector_number: raw.sector_number(),
                    });
                }
            }
            DefectList::LongBlockFormat(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::LongBlockFormatAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw =
                        super::format_unit::LongBlockFormatAddressDescriptor::from_bytes(bytes);
                    v.push(LongBlockFormatAddressDescriptor {
                        long_block_address: raw.long_block_address(),
                    });
                }
            }
            DefectList::BytesFromIndexFormat(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::BytesFromIndexFormatAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw = super::format_unit::BytesFromIndexFormatAddressDescriptor::from_bytes(
                        bytes,
                    );
                    v.push(BytesFromIndexFormatAddressDescriptor {
                        cylinder_number: raw.cylinder_number(),
                        head_number: raw.head_number(),
                        bytes_from_index: raw.bytes_from_index(),
                    });
                }
            }
            DefectList::PhysicalSectorFormat(v) => {
                for chunk in unsafe { result.data.elements_as_slice() }.chunks(size_of::<
                    super::format_unit::PhysicalSectorFormatAddressDescriptor,
                >()) {
                    let (bytes, _) = get_array(chunk);
                    let raw = super::format_unit::PhysicalSectorFormatAddressDescriptor::from_bytes(
                        bytes,
                    );
                    v.push(PhysicalSectorFormatAddressDescriptor {
                        cylinder_number: raw.cylinder_number(),
                        head_number: raw.head_number(),
                        sector_number: raw.sector_number(),
                    });
                }
            }
            DefectList::Custom(v) => {
                v.extend_from_slice(unsafe { result.data.elements_as_slice() });
            }
        }

        Ok((
            unsafe { result.data.get_body_maybe_uninit().assume_init() },
            defect_list,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_10: usize = 10;
    const COMMAND_LENGTH_12: usize = 12;
    const DATA_HEADER_LENGTH_10: usize = 4;
    const DATA_HEADER_LENGTH_12: usize = 8;

    #[test]
    fn layout_test() {
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
            size_of::<DataBufferHeader10>(),
            DATA_HEADER_LENGTH_10,
            concat!("Size of: ", stringify!(DataBufferHeader10))
        );

        assert_eq!(
            size_of::<DataBufferHeader12>(),
            DATA_HEADER_LENGTH_12,
            concat!("Size of: ", stringify!(DataBufferHeader12))
        );
    }
}
