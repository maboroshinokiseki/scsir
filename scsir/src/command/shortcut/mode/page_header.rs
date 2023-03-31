use modular_bitfield_msb::prelude::*;

use crate::command::get_array;

use super::ModePage;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct CommomPageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub page_length: B8,
}

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct CommomSubpageHeader {
    pub parameters_saveable: B1,
    pub subpage_format: B1,
    pub page_code: B6,
    pub subpage_code: B8,
    pub page_length: B16,
}

#[derive(Clone, Copy, Debug)]
pub enum PageHeaderStorage {
    Page(CommomPageHeader),
    Subpage(CommomSubpageHeader),
}

impl PageHeaderStorage {
    pub fn parameters_saveable(&self) -> bool {
        match self {
            PageHeaderStorage::Page(p) => p.parameters_saveable() != 0,
            PageHeaderStorage::Subpage(p) => p.parameters_saveable() != 0,
        }
    }

    pub fn subpage_format(&self) -> bool {
        match self {
            PageHeaderStorage::Page(p) => p.subpage_format() != 0,
            PageHeaderStorage::Subpage(p) => p.subpage_format() != 0,
        }
    }

    pub fn page_code(&self) -> u8 {
        match self {
            PageHeaderStorage::Page(p) => p.page_code(),
            PageHeaderStorage::Subpage(p) => p.page_code(),
        }
    }

    pub fn subpage_code(&self) -> u8 {
        match self {
            PageHeaderStorage::Page(_) => 0,
            PageHeaderStorage::Subpage(p) => p.subpage_code(),
        }
    }

    pub fn page_length(&self) -> u16 {
        match self {
            PageHeaderStorage::Page(p) => p.page_length() as u16,
            PageHeaderStorage::Subpage(p) => p.page_length(),
        }
    }
}

impl ModePage for PageHeaderStorage {
    fn new() -> Self {
        Self::Page(CommomPageHeader::new())
    }

    fn from_bytes(bytes: &[u8]) -> (Self, &[u8]) {
        let (array, mut left) = get_array(bytes);

        let page_header = CommomPageHeader::from_bytes(array);
        let page_header = if page_header.subpage_format() == 0 {
            PageHeaderStorage::Page(page_header)
        } else {
            let array;
            (array, left) = get_array(bytes);
            PageHeaderStorage::Subpage(CommomSubpageHeader::from_bytes(array))
        };

        (page_header, left)
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            PageHeaderStorage::Page(p) => p.bytes.to_vec(),
            PageHeaderStorage::Subpage(p) => p.bytes.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const PAGE_HEADER_LENGTH: usize = 2;
    const SUBPAGE_HEADER_LENGTH: usize = 4;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommomPageHeader>(),
            PAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(CommomPageHeader))
        );

        assert_eq!(
            size_of::<CommomSubpageHeader>(),
            SUBPAGE_HEADER_LENGTH,
            concat!("Size of: ", stringify!(CommomSubpageHeader))
        );
    }
}
