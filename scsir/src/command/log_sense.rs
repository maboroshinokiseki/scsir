#![allow(dead_code)]

use std::{
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
};

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct LogSenseCommand<'a> {
    interface: &'a Scsi,
    page_control: u8,
    page_code: u8,
    command_buffer: CommandBuffer,
}

impl<'a> LogSenseCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
            page_control: 0,
            page_code: 0,
        }
    }

    pub fn saving_parameters(&mut self, value: bool) -> &mut Self {
        self.command_buffer.set_saving_parameters(value as u8);
        self
    }

    // page_control must be less than 0x04
    pub fn page_control(&mut self, value: u8) -> &mut Self {
        self.page_control = value;
        self
    }

    // page_code must be less than 0x40
    pub fn page_code(&mut self, value: u8) -> &mut Self {
        self.page_code = value;
        self
    }

    pub fn subpage_code(&mut self, value: u8) -> &mut Self {
        self.command_buffer.set_subpage_code(value);
        self
    }

    pub fn parameter_pointer(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_parameter_pointer(value);
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

    pub fn issue(&mut self) -> crate::Result<Vec<u8>> {
        let result: FlexibleStruct<(), u8> =
            self.issue_flex(self.command_buffer.allocation_length() as usize)?;

        unsafe { Ok(result.elements_as_slice().to_vec()) }
    }

    pub fn issue_generic<Body: Copy, Element: Copy>(
        &mut self,
        element_length: usize,
    ) -> crate::Result<(MaybeUninit<Body>, Vec<MaybeUninit<Element>>)> {
        let result: FlexibleStruct<Body, Element> = self.issue_flex(element_length)?;

        Ok((
            result.get_body_maybe_uninit(),
            result.iter_clone().map(|e| MaybeUninit::new(e)).collect(),
        ))
    }

    pub(crate) fn issue_flex<B: Copy, E: Copy>(
        &mut self,
        element_length: usize,
    ) -> crate::Result<FlexibleStruct<B, E>> {
        let max_element = (u16::MAX as usize - size_of::<B>()) / size_of::<E>();
        if element_length > max_element {
            return Err(
                crate::Error::ArgumentOutOfBounds(
                    format!(
                        "Expected element length is out of bounds. The maximum possible value is {}, but {} was provided.",
                        max_element,
                        element_length)));
        }

        bitfield_bound_check!(self.page_control, 2, "page control")?;
        bitfield_bound_check!(self.page_code, 6, "page code")?;

        let temp = ThisCommand {
            command_buffer: self
                .command_buffer
                .with_page_control(self.page_control)
                .with_page_code(self.page_code),
            element_length,
            phantom_data: PhantomData,
        };

        self.interface.issue(&temp)
    }
}

impl Scsi {
    pub fn log_sense(&self) -> LogSenseCommand {
        LogSenseCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x4D;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved_0: B6,
    obsolete: B1,
    saving_parameters: B1,
    page_control: B2,
    page_code: B6,
    subpage_code: B8,
    reserved_1: B8,
    parameter_pointer: B16,
    allocation_length: B16,
    control: B8,
}

struct ThisCommand<Body, Element> {
    command_buffer: CommandBuffer,
    element_length: usize,

    phantom_data: PhantomData<(Body, Element)>,
}

impl<Body: Copy, Element: Copy> Command for ThisCommand<Body, Element> {
    type CommandBuffer = CommandBuffer;

    type DataBuffer = AnyType;

    type DataBufferWrapper = FlexibleStruct<Body, Element>;

    type ReturnType = crate::Result<FlexibleStruct<Body, Element>>;

    fn direction(&self) -> DataDirection {
        DataDirection::FromDevice
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
            .with_allocation_length(self.data_size().try_into().unwrap_or(u16::MAX))
    }

    fn data(&self) -> Self::DataBufferWrapper {
        unsafe { FlexibleStruct::with_length(self.element_length) }
    }

    fn data_size(&self) -> u32 {
        (size_of::<Body>() + self.element_length * size_of::<Element>()) as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        Ok(result.data.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH: usize = 10;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
