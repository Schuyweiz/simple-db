use anyhow::Result;

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

    pub fn get_page_mut(&mut self) -> &mut Pager {
        &mut self.pager
    }

    pub fn get_current_row_count(&self) -> usize {
        self.current_rows
    }

    pub fn flush(&mut self) -> Result<()> {
        self.pager.flush()
    }

    pub fn increment_current_row_count(&mut self) {
        self.current_rows += 1;
    }
}
