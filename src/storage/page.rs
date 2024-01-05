use crate::storage::constant::{PAGE_SIZE, ROWS_PER_PAGE, ROW_SIZE};

#[derive(Clone, Copy, Debug)]
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new() -> Page {
        Page {
            data: [0; PAGE_SIZE],
        }
    }

    pub fn get_page_data(&self) -> &[u8] {
        &self.data
    }

    pub fn deserialize(data: &[u8]) -> Page {
        let mut page = Page::new();
        page.data.copy_from_slice(data);
        page
    }

    pub fn get_slot(&mut self, row_index: usize) -> &mut [u8] {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        &mut self.data[byte_offset..byte_offset + ROW_SIZE]
    }
}
