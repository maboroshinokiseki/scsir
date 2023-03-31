#![allow(dead_code)]

use super::{ModePage, PageHeaderStorage};

#[derive(Clone, Debug)]
pub struct GeneralPage {
    header: PageHeaderStorage,
    body: Vec<u8>,
}

impl ModePage for GeneralPage {
    fn new() -> Self {
        Self {
            header: PageHeaderStorage::new(),
            body: vec![],
        }
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (header, bytes) = PageHeaderStorage::from_bytes(bytes);

        let (body, left) = bytes.split_at(usize::min(bytes.len(), header.page_length() as usize));

        (
            Self {
                header,
                body: body.to_vec(),
            },
            left,
        )
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&self.body);

        bytes
    }
}
