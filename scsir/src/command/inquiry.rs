#![allow(dead_code)]

use std::{
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
};

use modular_bitfield_msb::prelude::*;

use crate::{
    data_wrapper::{AnyType, FlexibleStruct},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

#[derive(Clone, Debug)]
pub struct InquiryCommand<'a> {
    interface: &'a Scsi,
    command_buffer: CommandBuffer,
}

impl<'a> InquiryCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            command_buffer: CommandBuffer::new().with_operation_code(OPERATION_CODE),
        }
    }

    pub fn page_code(&mut self, value: Option<u8>) -> &mut Self {
        self.command_buffer.set_page_code(value.unwrap_or(0));
        self.command_buffer
            .set_enable_vital_product_data(value.is_some() as u8);
        self
    }

    pub fn allocation_length(&mut self, value: u16) -> &mut Self {
        self.command_buffer.set_allocation_length(value);
        self
    }

    pub fn issue(&mut self) -> crate::Result<Vec<u8>> {
        let result: FlexibleStruct<(), u8> =
            self.issue_flex(self.command_buffer.allocation_length().into())?;

        unsafe { Ok(result.elements_as_slice().to_vec()) }
    }

    pub fn issue_generic<Body: Copy, Element: Copy>(
        &mut self,
        element_length: usize,
    ) -> crate::Result<(MaybeUninit<Body>, Vec<MaybeUninit<Element>>)> {
        let result: FlexibleStruct<Body, Element> = self.issue_flex(element_length)?;

        Ok((
            result.get_body_maybe_uninit(),
            result.iter_maybe_uninit().collect(),
        ))
    }

    pub(crate) fn issue_flex<B: Copy, E: Copy>(
        &self,
        element_length: usize,
    ) -> crate::Result<FlexibleStruct<B, E>> {
        let max_element = (u16::MAX as usize - size_of::<B>()) / usize::max(size_of::<E>(), 1);
        if element_length > max_element {
            return Err(
                crate::Error::ArgumentOutOfBounds(
                    format!(
                        "Expected element length is out of bounds. The maximum possible value is {}, but {} was provided.",
                        max_element,
                        element_length)));
        }

        let this_command: ThisCommand<B, E> = ThisCommand {
            command_buffer: self.command_buffer,
            element_length,
            phantom_data: PhantomData,
        };

        self.interface.issue(&this_command)
    }
}

impl Scsi {
    pub fn inquiry(&self) -> InquiryCommand {
        InquiryCommand::new(self)
    }
}

const OPERATION_CODE: u8 = 0x12;

#[bitfield]
#[derive(Clone, Copy, Debug)]
struct CommandBuffer {
    operation_code: B8,
    reserved: B6,
    obsolete_command_support_data: B1,
    enable_vital_product_data: B1,
    page_code: B8,
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

    const COMMAND_LENGTH: usize = 6;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer>(),
            COMMAND_LENGTH,
            concat!("Size of: ", stringify!(CommandBuffer))
        );
    }
}
