use anyhow::Result;

use crate::storage::pager::Pager;
use crate::storage::row::Row;

pub struct Table {
    root_page_num: usize,
    pager: Pager,
}

impl Table {
    pub fn open_db_connection(file_path: &str) -> Result<Self> {
        let mut pager = Pager::new(file_path).unwrap();
        let root_page_num = pager.get_node_mut(0).get_parent_page_num();
        Ok(Self {
            root_page_num,
            pager,
        })
    }

    pub fn insert(&mut self, page_num: usize, cell_num: usize, row: &Row) {
        self.pager
            .insert(
                cell_num.to_le_bytes().as_slice(),
                &row.serialize().unwrap(),
                page_num,
            )
            .expect("TODO: panic message");
    }

    pub fn select(&mut self, page_num: usize, cell_num: usize) -> &[u8] {
        self.pager.select(page_num, cell_num)
    }

    pub fn get_page_mut(&mut self) -> &mut Pager {
        &mut self.pager
    }

    pub fn get_root_page_num(&self) -> usize {
        self.root_page_num
    }

    pub fn flush(&mut self) -> Result<()> {
        self.pager.flush()
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;

    #[test]
    fn test_table() {
        let test_db_path = "test.db";
        let mut table = Table::open_db_connection(test_db_path).unwrap();
        let row = Row::new(1, "test".to_string(), "test".to_string());
        table.insert(0, 0, &row);
        table.flush().unwrap();

        let mut table = Table::open_db_connection(test_db_path).unwrap();
        let row = Row::deserialize(table.select(0, 0)).unwrap();
        assert_eq!(row.get_id(), 1);
        assert_eq!(row.get_user_name(), "test");
        assert_eq!(row.get_email(), "test");

        fs::remove_file(test_db_path).unwrap();
    }
}
