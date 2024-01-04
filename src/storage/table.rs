use crate::storage::constant::{TABLE_MAX_PAGES, ROWS_PER_PAGE};
use crate::storage::constant::{ROW_SIZE, PAGE_SIZE};

pub struct Table {
    current_rows: usize,
    pages: Vec<Option<Page>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            current_rows: 0,
            pages: vec![None; TABLE_MAX_PAGES],
        }
    }

    pub fn get_page_mut(&mut self, page_num: usize) -> &mut Page {
        if self.pages[page_num].is_none() {
            self.pages[page_num] = Some(Page::new());
        }

        self.pages[page_num].as_mut().unwrap()
    }

    pub fn insert(&mut self, data: &[u8]) {
        let current_row_count = self.get_current_row_count();
        let page_num = current_row_count / ROWS_PER_PAGE;
        let page = self.get_page_mut(page_num);

        page.write_slot(current_row_count, data);
        self.increment_current_row_count();
    }

    pub fn select(&self, row_index: usize) -> Option<&[u8]> {
        let page_num = row_index / ROWS_PER_PAGE;
        let page = self.get_page_ref(page_num);

        page.map(|page| page.read_slot(row_index))
    }

    // returns an optional reference to a page
    fn get_page_ref(&self, page_num: usize) -> Option<&Page> {
        self.pages[page_num].as_ref()
    }

    fn increment_current_row_count(&mut self) {
        self.current_rows += 1;
    }

    // incapsulate current_rows
    pub fn get_current_row_count(&self) -> usize {
        self.current_rows
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new() -> Page {
        Page { data: [0; PAGE_SIZE] }
    }

    pub fn read_slot(&self, row_index: usize) -> &[u8] {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        &self.data[byte_offset..byte_offset + ROW_SIZE]
    }

    pub fn write_slot(&mut self, row_index: usize, bytes: &[u8]) {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        self.data[byte_offset..byte_offset + ROW_SIZE].copy_from_slice(bytes);
    }
}
