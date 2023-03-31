use super::{DescriptorStorage, DescriptorType, HeaderStorage, HeaderType};

#[derive(Clone, Debug)]
pub struct PageWrapper<Page: ModePage> {
    pub header: HeaderStorage,
    pub descriptors: Vec<DescriptorStorage>,
    pub page: Page,
}

pub trait ModePage {
    fn new() -> Self
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8])
    where
        Self: Sized;

    fn to_bytes(&self) -> Vec<u8>;
}

impl<Page: ModePage> PageWrapper<Page> {
    pub fn from_bytes(
        header_type: HeaderType,
        descriptor_type: DescriptorType,
        bytes: &[u8],
    ) -> Self {
        let (header, bytes) = HeaderStorage::from_bytes(header_type, bytes);

        let block_descriptor_length = header.block_descriptor_length();

        let mut descriptors = vec![];
        let (mut descriptor_bytes, bytes) =
            bytes.split_at(usize::min(block_descriptor_length as usize, bytes.len()));

        while !descriptor_bytes.is_empty() {
            let descriptor;
            (descriptor, descriptor_bytes) =
                DescriptorStorage::from_bytes(descriptor_type, descriptor_bytes);
            descriptors.push(descriptor);
        }

        Self {
            header,
            descriptors,
            page: Page::from_bytes(bytes).0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.to_bytes());

        for item in &self.descriptors {
            bytes.extend_from_slice(&item.to_bytes());
        }

        bytes.extend_from_slice(&self.page.to_bytes());

        bytes
    }
}
