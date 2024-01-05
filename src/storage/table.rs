use anyhow::Result;

use crate::storage::constant::ROWS_PER_PAGE;
use crate::storage::pager::Pager;

pub struct Table {
    current_rows: usize,
    pager: Pager,
}

impl Table {
    pub fn open_db_connection(file_path: &str) -> Result<Self> {
        let pager = Pager::new(file_path).unwrap();
        Ok(Self {
            current_rows: pager.get_rows_count(),
            pager,
        })
    }

    pub fn insert(&mut self, data: &[u8]) {
        let current_row_count = self.get_current_row_count();
        let page_num = current_row_count / ROWS_PER_PAGE;
        let page = self.pager.get_page_mut(page_num);

        page.write_to_slot(current_row_count, data);
        self.increment_current_row_count();
    }

    pub fn select(&mut self, row_index: usize) -> &[u8] {
        let page_num = row_index / ROWS_PER_PAGE;
        let page = self.pager.get_page_mut(page_num);

        page.read_from_slot(row_index)
    }

    pub fn get_current_row_count(&self) -> usize {
        self.current_rows
    }

    pub fn flush(&mut self) -> Result<()> {
        self.pager.flush()
    }

    fn increment_current_row_count(&mut self) {
        self.current_rows += 1;
    }
}
