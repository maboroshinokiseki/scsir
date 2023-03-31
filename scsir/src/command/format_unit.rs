#![allow(dead_code)]

use std::mem::size_of_val;

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct FormatUnitCommand<'a> {
    interface: &'a Scsi,
    format_protection_information: u8,
    defect_list_format: u8,
    fast_format: u8,
    command_buffer: CommandBuffer,
    header_buffer: LongParameterListHeader,
    initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader,
    initialization_pattern: Vec<u8>,
    defect_list: Vec<DefectListItem>,
}

pub struct ParameterBuilder<'a> {
    parent: &'a mut FormatUnitCommand<'a>,
    longlist: bool,
    header_buffer: LongParameterListHeader,
    initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader,
    initialization_pattern: Vec<u8>,
    defect_list: Vec<DefectListItem>,
}

pub struct ShortParameterListHeaderBuilder<'a> {
    delegate: LongParameterListHeaderBuilder<'a>,
}

pub struct LongParameterListHeaderBuilder<'a> {
    parent: &'a mut ParameterBuilder<'a>,
    protection_fields_usage: u8,
    protection_interval_exponent: u8,
    header_buffer: LongParameterListHeader,
}

pub struct InitializationPatternDescriptorBuilder<'a> {
    parent: &'a mut ParameterBuilder<'a>,
    initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader,
    initialization_pattern: Vec<u8>,
}
pub struct DefectListBuilder<'a> {
    parent: &'a mut ParameterBuilder<'a>,
    defect_list: Vec<DefectListItem>,
}

impl<'a> FormatUnitCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            format_protection_information: 0,
            defect_list_format: 0,
            fast_format: 0,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            header_buffer: LongParameterListHeader::new(),
            initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader::new(),
            initialization_pattern: vec![],
            defect_list: vec![],
        }
    }

    // format_protection_information must be less than 0x4
    pub fn format_protection_information(&mut self, value: u8) -> &mut Self {
        self.format_protection_information = value;
        self
    }

    pub fn format_data(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_format_data(value as u8);
        self
    }

    pub fn complete_list(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_complete_list(value as u8);
        self
    }

    // defect_list_format must be less than 0x8
    pub fn defect_list_format(&mut self, value: u8) -> &mut Self {
        self.defect_list_format = value;
        self
    }

    pub fn vendor_specific(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_vendor_specific(value);
        self
    }

    // fast_format must be less than 0x4
    pub fn fast_format(&mut self, value: u8) -> &mut Self {
        self.fast_format = value;
        self
    }

    pub fn control(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_control(value);
        self
    }

    pub fn parameter(&'a mut self) -> ParameterBuilder {
        ParameterBuilder::new(self)
    }

    pub fn issue(&mut self) -> crate::Result<()> {
        bitfield_bound_check!(
            self.format_protection_information,
            2,
            "format protection information"
        )?;
        bitfield_bound_check!(self.defect_list_format, 3, "defect list format")?;
        bitfield_bound_check!(self.fast_format, 2, "fast format")?;
        self.command_buffer
            .set_format_protection_information(self.format_protection_information);
        self.command_buffer
            .set_defect_list_format(self.defect_list_format);
        self.command_buffer.set_fast_format(self.fast_format);

        if self.command_buffer.format_data() == 0 {
            let temp = ThisCommand {
                command_buffer: self.command_buffer,
                data_buffer: vec![],
            };
            return self.interface.issue(&temp);
        }

        let mut data_buffer: Vec<u8> = vec![];

        if self.command_buffer.longlist() == 0 {
            let long_list = self.header_buffer;

            bitfield_bound_check!(long_list.defect_list_length(), 16, "defect list length")?;

            let header = ShortParameterListHeader::new()
                .with_protection_fields_usage(long_list.protection_fields_usage())
                .with_format_options_valid(long_list.format_options_valid())
                .with_disable_primary(long_list.disable_primary())
                .with_disable_certification(long_list.disable_certification())
                .with_stop_format(long_list.stop_format())
                .with_initialization_pattern(long_list.initialization_pattern())
                .with_immediate(long_list.immediate())
                .with_vendor_specific(long_list.vendor_specific())
                .with_defect_list_length(long_list.defect_list_length() as u16);

            data_buffer.extend_from_slice(&header.bytes);
        } else {
            data_buffer.extend_from_slice(&self.header_buffer.bytes);
        }

        if self.header_buffer.initialization_pattern() == 1 {
            data_buffer.extend_from_slice(&self.initialization_pattern_descriptor_header.bytes);
            data_buffer.extend_from_slice(&self.initialization_pattern);
        }

        for item in &self.defect_list {
            match item {
                DefectListItem::ShortBlockFormatAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::ExtendedBytesFromIndexAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::ExtendedPhysicalSectorAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::LongBlockFormatAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::BytesFromIndexFormatAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::PhysicalSectorFormatAddressDescriptor(x) => {
                    data_buffer.extend_from_slice(&x.bytes)
                }
                DefectListItem::CustomDescriptor(x) => data_buffer.extend_from_slice(x),
            }
        }

        let temp = ThisCommand {
            command_buffer: self.command_buffer,
            data_buffer,
        };
        self.interface.issue(&temp)
    }
}

impl<'a> ParameterBuilder<'a> {
    fn new(parent: &'a mut FormatUnitCommand<'a>) -> Self {
        Self {
            parent,
            longlist: false,
            header_buffer: LongParameterListHeader::new(),
            initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader::new(),
            initialization_pattern: vec![],
            defect_list: vec![],
        }
    }

    pub fn short_parameter_list_header(&'a mut self) -> ShortParameterListHeaderBuilder<'a> {
        ShortParameterListHeaderBuilder::new(self)
    }

    pub fn long_parameter_list_header(&'a mut self) -> LongParameterListHeaderBuilder<'a> {
        LongParameterListHeaderBuilder::new(self)
    }

    pub fn initialization_pattern_descriptor(
        &'a mut self,
    ) -> InitializationPatternDescriptorBuilder<'a> {
        InitializationPatternDescriptorBuilder::new(self)
    }

    pub fn defect_list(&'a mut self) -> DefectListBuilder<'a> {
        DefectListBuilder::new(self)
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut FormatUnitCommand> {
        self.parent
            .command_buffer
            .set_longlist(self.longlist.into());
        self.parent.header_buffer = self.header_buffer;
        self.parent.initialization_pattern_descriptor_header =
            self.initialization_pattern_descriptor_header;
        self.parent.initialization_pattern = std::mem::take(&mut self.initialization_pattern);
        self.parent.defect_list = std::mem::take(&mut self.defect_list);
        Ok(self.parent)
    }
}

impl<'a> ShortParameterListHeaderBuilder<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            delegate: parent.long_parameter_list_header(),
        }
    }

    // protection_fields_usage must be less than 0x8
    pub fn protection_fields_usage(&mut self, value: u8) -> &mut Self {
        self.delegate.protection_fields_usage(value);
        self
    }

    pub fn format_options_valid(&mut self, value: bool) -> &mut Self {
        self.delegate.format_options_valid(value);
        self
    }

    pub fn disable_primary(&mut self, value: bool) -> &mut Self {
        self.delegate.disable_primary(value);
        self
    }

    pub fn disable_certification(&mut self, value: bool) -> &mut Self {
        self.delegate.disable_certification(value);
        self
    }

    pub fn stop_format(&mut self, value: bool) -> &mut Self {
        self.delegate.stop_format(value);
        self
    }

    pub fn initialization_pattern(&mut self, value: bool) -> &mut Self {
        self.delegate.initialization_pattern(value);
        self
    }

    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.delegate.immediate(value);
        self
    }

    pub fn vendor_specific(&mut self, value: bool) -> &mut Self {
        self.delegate.vendor_specific(value);
        self
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut ParameterBuilder> {
        let parent = self.delegate.done()?;
        parent.longlist = false;
        Ok(parent)
    }
}

impl<'a> LongParameterListHeaderBuilder<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            parent,
            protection_fields_usage: 0,
            protection_interval_exponent: 0,
            header_buffer: LongParameterListHeader::new(),
        }
    }

    // protection_fields_usage must be less than 0x8
    pub fn protection_fields_usage(&mut self, value: u8) -> &mut Self {
        self.protection_fields_usage = value;
        self
    }

    pub fn format_options_valid(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_format_options_valid(value as u8);
        self
    }

    pub fn disable_primary(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_disable_primary(value as u8);
        self
    }

    pub fn disable_certification(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_disable_certification(value as u8);
        self
    }

    pub fn stop_format(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_stop_format(value as u8);
        self
    }

    pub fn initialization_pattern(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_initialization_pattern(value as u8);
        self
    }

    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_immediate(value as u8);
        self
    }

    pub fn vendor_specific(&mut self, value: bool) -> &mut Self {
        self.header_buffer.set_vendor_specific(value as u8);
        self
    }

    // protection_fields_usage must be less than 0x10
    pub fn protection_interval_exponent(&mut self, value: u8) -> &mut Self {
        self.protection_interval_exponent = value;
        self
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut ParameterBuilder> {
        bitfield_bound_check!(self.protection_fields_usage, 3, "protection fields usage")?;
        bitfield_bound_check!(
            self.protection_interval_exponent,
            4,
            "protection interval exponent"
        )?;

        self.header_buffer
            .set_protection_fields_usage(self.protection_fields_usage);
        self.header_buffer
            .set_protection_interval_exponent(self.protection_interval_exponent);

        self.parent.header_buffer = self.header_buffer;
        self.parent.longlist = true;

        Ok(self.parent)
    }
}

impl<'a> InitializationPatternDescriptorBuilder<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            parent,
            initialization_pattern_descriptor_header: InitializationPatternDescriptorHeader::new(),
            initialization_pattern: vec![],
        }
    }

    pub fn security_initialize(&mut self, value: bool) -> &mut Self {
        self.initialization_pattern_descriptor_header
            .set_security_initialize(value as u8);
        self
    }

    pub fn initialization_pattern_type(&mut self, value: u8) -> &mut Self {
        self.initialization_pattern_descriptor_header
            .set_initialization_pattern_type(value);
        self
    }

    // initialization_pattern length must be less or equal than 0xFFFF
    pub fn initialization_pattern(&mut self, value: &[u8]) -> &mut Self {
        self.initialization_pattern.extend_from_slice(value);
        self
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut ParameterBuilder> {
        bitfield_bound_check!(
            self.initialization_pattern.len(),
            16,
            "initialization pattern length"
        )?;
        self.initialization_pattern_descriptor_header
            .set_initialization_pattern_length(self.initialization_pattern.len() as u16);
        self.parent.initialization_pattern_descriptor_header =
            self.initialization_pattern_descriptor_header;
        self.parent.initialization_pattern = std::mem::take(&mut self.initialization_pattern);

        Ok(self.parent)
    }
}

impl<'a> DefectListBuilder<'a> {
    fn new(parent: &'a mut ParameterBuilder<'a>) -> Self {
        Self {
            parent,
            defect_list: vec![],
        }
    }

    pub fn add_short_block_format_address_descriptor(
        &mut self,
        short_block_address: u32,
    ) -> &mut Self {
        self.defect_list
            .push(DefectListItem::ShortBlockFormatAddressDescriptor(
                ShortBlockFormatAddressDescriptor::new()
                    .with_short_block_address(short_block_address),
            ));
        self
    }

    // cylinder_number must be less or qual than 0xFF_FFFF
    // bytes_from_index must be less or qual than 0xFFF_FFFF
    pub fn add_extended_bytes_from_index_address_descriptor_checked(
        &mut self,
        cylinder_number: u32,
        head_number: u8,
        multi_address_descriptor_start: bool,
        bytes_from_index: u32,
    ) -> crate::Result<&mut Self> {
        bitfield_bound_check!(cylinder_number, 24, "cylinder number")?;
        bitfield_bound_check!(bytes_from_index, 28, "bytes from index")?;

        self.defect_list
            .push(DefectListItem::ExtendedBytesFromIndexAddressDescriptor(
                ExtendedBytesFromIndexAddressDescriptor::new()
                    .with_cylinder_number(cylinder_number)
                    .with_head_number(head_number)
                    .with_multi_address_descriptor_start(multi_address_descriptor_start.into())
                    .with_bytes_from_index(bytes_from_index),
            ));

        Ok(self)
    }

    // cylinder_number must be less or qual than 0xFF_FFFF
    // sector_number must be less or qual than 0xFFF_FFFF
    pub fn add_extended_physical_sector_address_descriptor_checked(
        &mut self,
        cylinder_number: u32,
        head_number: u8,
        multi_address_descriptor_start: bool,
        sector_number: u32,
    ) -> crate::Result<&mut Self> {
        bitfield_bound_check!(cylinder_number, 24, "cylinder number")?;
        bitfield_bound_check!(sector_number, 28, "sector number")?;

        self.defect_list
            .push(DefectListItem::ExtendedPhysicalSectorAddressDescriptor(
                ExtendedPhysicalSectorAddressDescriptor::new()
                    .with_cylinder_number(cylinder_number)
                    .with_head_number(head_number)
                    .with_multi_address_descriptor_start(multi_address_descriptor_start.into())
                    .with_sector_number(sector_number),
            ));

        Ok(self)
    }

    pub fn add_long_block_format_address_descriptor(
        &mut self,
        long_block_address: u64,
    ) -> &mut Self {
        self.defect_list
            .push(DefectListItem::LongBlockFormatAddressDescriptor(
                LongBlockFormatAddressDescriptor::new().with_long_block_address(long_block_address),
            ));
        self
    }

    // cylinder_number must be less or qual than 0xFF_FFFF
    pub fn add_bytes_from_index_format_address_descriptor_checked(
        &mut self,
        cylinder_number: u32,
        head_number: u8,
        bytes_from_index: u32,
    ) -> crate::Result<&mut Self> {
        bitfield_bound_check!(cylinder_number, 24, "cylinder number")?;

        self.defect_list
            .push(DefectListItem::BytesFromIndexFormatAddressDescriptor(
                BytesFromIndexFormatAddressDescriptor::new()
                    .with_cylinder_number(cylinder_number)
                    .with_head_number(head_number)
                    .with_bytes_from_index(bytes_from_index),
            ));

        Ok(self)
    }

    // cylinder_number must be less or qual than 0xFF_FFFF
    pub fn add_physical_sector_format_address_descriptor_checked(
        &mut self,
        cylinder_number: u32,
        head_number: u8,
        sector_number: u32,
    ) -> crate::Result<&mut Self> {
        bitfield_bound_check!(cylinder_number, 24, "cylinder number")?;

        self.defect_list
            .push(DefectListItem::PhysicalSectorFormatAddressDescriptor(
                PhysicalSectorFormatAddressDescriptor::new()
                    .with_cylinder_number(cylinder_number)
                    .with_head_number(head_number)
                    .with_sector_number(sector_number),
            ));

        Ok(self)
    }

    pub fn add_custom_descriptor(&mut self, descriptor: &[u8]) -> &mut Self {
        self.defect_list
            .push(DefectListItem::CustomDescriptor(Vec::from(descriptor)));

        self
    }

    pub fn done(&'a mut self) -> crate::Result<&'a mut ParameterBuilder> {
        for w in self.defect_list.windows(2) {
            if std::mem::discriminant(&w[0]) != std::mem::discriminant(&w[1]) {
                return Err(crate::Error::BadArgument(
                    "Cannot mix descriptor types".to_owned(),
                ));
            }
        }

        let item_len = if let Some(item) = self.defect_list.first() {
            match item {
                DefectListItem::ShortBlockFormatAddressDescriptor(x) => size_of_val(x),
                DefectListItem::ExtendedBytesFromIndexAddressDescriptor(x) => size_of_val(x),
                DefectListItem::ExtendedPhysicalSectorAddressDescriptor(x) => size_of_val(x),
                DefectListItem::LongBlockFormatAddressDescriptor(x) => size_of_val(x),
                DefectListItem::BytesFromIndexFormatAddressDescriptor(x) => size_of_val(x),
                DefectListItem::PhysicalSectorFormatAddressDescriptor(x) => size_of_val(x),
                DefectListItem::CustomDescriptor(_) => 0,
            }
        } else {
            0
        };

        let total_len = if item_len == 0 {
            let mut total_len = 0;
            for item in &self.defect_list {
                if let DefectListItem::CustomDescriptor(x) = item {
                    total_len += x.len()
                }
            }

            total_len
        } else {
            item_len * self.defect_list.len()
        };

        let defect_list_length_bits = if self.parent.longlist { 32 } else { 16 };

        bitfield_bound_check!(total_len, defect_list_length_bits, "defect list length")?;

        self.parent
            .header_buffer
            .set_defect_list_length(total_len as u32);
        self.parent.defect_list = std::mem::take(&mut self.defect_list.clone());

        Ok(self.parent)
    }
}

impl Scsi {
    pub fn format_unit(&self) -> FormatUnitCommand {
        FormatUnitCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x04;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    format_protection_information: B2,
    longlist: B1,
    format_data: B1,
    complete_list: B1,
    defect_list_format: B3,
    vendor_specific: B8,
    reserved: B14,
    fast_format: B2,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct ShortParameterListHeader {
    reserved: B5,
    protection_fields_usage: B3,
    format_options_valid: B1,
    disable_primary: B1,
    disable_certification: B1,
    stop_format: B1,
    initialization_pattern: B1,
    obsolete: B1,
    immediate: B1,
    vendor_specific: B1,
    defect_list_length: B16,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct LongParameterListHeader {
    reserved_0: B5,
    protection_fields_usage: B3,
    format_options_valid: B1,
    disable_primary: B1,
    disable_certification: B1,
    stop_format: B1,
    initialization_pattern: B1,
    obsolete: B1,
    immediate: B1,
    vendor_specific: B1,
    reserved_1: B8,
    // should be zero
    p_i_information: B4,
    protection_interval_exponent: B4,
    defect_list_length: B32,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct InitializationPatternDescriptorHeader {
    obsolete: B2,
    security_initialize: B1,
    reserved: B5,
    initialization_pattern_type: B8,
    initialization_pattern_length: B16,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
enum DefectListItem {
    ShortBlockFormatAddressDescriptor(ShortBlockFormatAddressDescriptor),
    ExtendedBytesFromIndexAddressDescriptor(ExtendedBytesFromIndexAddressDescriptor),
    ExtendedPhysicalSectorAddressDescriptor(ExtendedPhysicalSectorAddressDescriptor),
    LongBlockFormatAddressDescriptor(LongBlockFormatAddressDescriptor),
    BytesFromIndexFormatAddressDescriptor(BytesFromIndexFormatAddressDescriptor),
    PhysicalSectorFormatAddressDescriptor(PhysicalSectorFormatAddressDescriptor),
    CustomDescriptor(Vec<u8>),
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct ShortBlockFormatAddressDescriptor {
    pub(super) short_block_address: B32,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct ExtendedBytesFromIndexAddressDescriptor {
    pub(super) cylinder_number: B24,
    pub(super) head_number: B8,
    pub(super) multi_address_descriptor_start: B1,
    reserved: B3,
    pub(super) bytes_from_index: B28,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct ExtendedPhysicalSectorAddressDescriptor {
    pub(super) cylinder_number: B24,
    pub(super) head_number: B8,
    pub(super) multi_address_descriptor_start: B1,
    reserved: B3,
    pub(super) sector_number: B28,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct LongBlockFormatAddressDescriptor {
    pub(super) long_block_address: B64,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct BytesFromIndexFormatAddressDescriptor {
    pub(super) cylinder_number: B24,
    pub(super) head_number: B8,
    pub(super) bytes_from_index: B32,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub(super) struct PhysicalSectorFormatAddressDescriptor {
    pub(super) cylinder_number: B24,
    pub(super) head_number: B8,
    pub(super) sector_number: B32,
}

struct ThisCommand {
    command_buffer: CommandBuffer,
    data_buffer: Vec<u8>,
}

impl Command for ThisCommand {
    type CommandBuffer = CommandBuffer;

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
        VecBufferWrapper(self.data_buffer.clone())
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

    const COMMAND_LENGTH: usize = 6;
    const SHORT_PARAMETER_LIST_HEADER_LENGTH: usize = 4;
    const LONG_PARAMETER_LIST_HEADER_LENGTH: usize = 8;
    const INITIALIZATION_PATTERN_DESCRIPTOR_HEADER_LENGTH: usize = 4;
    const SHORT_BLOCK_FORMAT_ADDRESS_DESCRIPTOR_LENGTH: usize = 4;
    const EXTENDED_BYTES_FROM_INDEX_ADDRESS_DESCRIPTOR_LENGTH: usize = 8;
    const EXTENDED_PHYSICAL_SECTOR_ADDRESS_DESCRIPTOR_LENGTH: usize = 8;
    const LONG_BLOCK_FORMAT_ADDRESS_DESCRIPTOR_LENGTH: usize = 8;
    const BYTES_FROM_INDEX_FORMAT_ADDRESS_DESCRIPTOR_LENGTH: usize = 8;
    const PHYSICAL_SECTOR_FORMAT_ADDRESS_DESCRIPTOR_LENGTH: usize = 8;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );

        assert_eq!(
            size_of::<ShortParameterListHeader>(),
            SHORT_PARAMETER_LIST_HEADER_LENGTH,
            concat!("Size of: ", stringify!(ShortParameterListHeader))
        );

        assert_eq!(
            size_of::<LongParameterListHeader>(),
            LONG_PARAMETER_LIST_HEADER_LENGTH,
            concat!("Size of: ", stringify!(LongParameterListHeader))
        );

        assert_eq!(
            size_of::<InitializationPatternDescriptorHeader>(),
            INITIALIZATION_PATTERN_DESCRIPTOR_HEADER_LENGTH,
            concat!(
                "Size of: ",
                stringify!(InitializationPatternDescriptorHeader)
            )
        );

        assert_eq!(
            size_of::<ShortBlockFormatAddressDescriptor>(),
            SHORT_BLOCK_FORMAT_ADDRESS_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(ShortBlockFormatAddressDescriptor))
        );

        assert_eq!(
            size_of::<ExtendedBytesFromIndexAddressDescriptor>(),
            EXTENDED_BYTES_FROM_INDEX_ADDRESS_DESCRIPTOR_LENGTH,
            concat!(
                "Size of: ",
                stringify!(ExtendedBytesFromIndexAddressDescriptor)
            )
        );

        assert_eq!(
            size_of::<ExtendedPhysicalSectorAddressDescriptor>(),
            EXTENDED_PHYSICAL_SECTOR_ADDRESS_DESCRIPTOR_LENGTH,
            concat!(
                "Size of: ",
                stringify!(ExtendedPhysicalSectorAddressDescriptor)
            )
        );

        assert_eq!(
            size_of::<LongBlockFormatAddressDescriptor>(),
            LONG_BLOCK_FORMAT_ADDRESS_DESCRIPTOR_LENGTH,
            concat!("Size of: ", stringify!(LongBlockFormatAddressDescriptor))
        );

        assert_eq!(
            size_of::<BytesFromIndexFormatAddressDescriptor>(),
            BYTES_FROM_INDEX_FORMAT_ADDRESS_DESCRIPTOR_LENGTH,
            concat!(
                "Size of: ",
                stringify!(BytesFromIndexFormatAddressDescriptor)
            )
        );

        assert_eq!(
            size_of::<PhysicalSectorFormatAddressDescriptor>(),
            PHYSICAL_SECTOR_FORMAT_ADDRESS_DESCRIPTOR_LENGTH,
            concat!(
                "Size of: ",
                stringify!(PhysicalSectorFormatAddressDescriptor)
            )
        );
    }
}
