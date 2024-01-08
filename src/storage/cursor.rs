use crate::storage::node::NodeType;
use crate::storage::row::Row;
use crate::storage::table::Table;

pub struct Cursor<'a> {
    table: &'a mut Table,
    page_num: usize,
    cell_num: usize,
    end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn table_start(table: &'a mut Table) -> Cursor {
        let end_of_table = table.get_pager_mut().get_node_mut(0).get_cell_count() == 0;
        Cursor {
            table,
            page_num: 0,
            cell_num: 0,
            end_of_table,
        }
    }

    pub fn new(table: &'a mut Table, page_num: usize, cell_num: usize) -> Cursor {
        let root_page_num = table.get_root_page_num();
        let end_of_table = table.get_pager_mut().get_node_mut(root_page_num).get_cell_count() == cell_num;
        Cursor {
            table,
            page_num,
            cell_num,
            end_of_table,
        }
    }

    pub fn table_find(table: &'a mut Table, key: usize) -> Cursor {
        let root_page_num = table.get_root_page_num();
        let root_node = table.get_pager_mut().get_node_mut(root_page_num);

        let root_node_type = root_node.get_node_type();
        if root_node_type == NodeType::Leaf {
            return Cursor::leaf_node_find(table, root_page_num, key);
        } else {
            return Cursor::internal_node_find(table, root_page_num, key);
        }
    }

    fn internal_node_find(table: &'a mut Table, page_num: usize, key: usize) -> Cursor {
        let node = table.get_pager_mut().get_node_mut(page_num);
        let keys_num = node.get_key_count();

        let mut min_index = 0;
        let mut max_index = keys_num;

        while min_index != max_index {
            let index = (min_index + max_index) / 2;
            let key_at_index = node.internal_get_key(index);

            if key_at_index >= key {
                max_index = index;
            } else {
                min_index = index + 1;
            }
        }

        let child_page_num = node.internal_node_children(min_index);
        let child_node = table.get_pager_mut().get_node_mut(child_page_num);
        let child_node_type = child_node.get_node_type();

        return match child_node_type {
            NodeType::Leaf => {
                Cursor::leaf_node_find(table, child_page_num, key)
            }
            NodeType::Internal => {
                Cursor::internal_node_find(table, child_page_num, key)
            }
        };
    }

    fn leaf_node_find(table: &'a mut Table, page_num: usize, key: usize) -> Cursor {
        let root_node = table.get_pager_mut().get_node_mut(page_num);

        let mut min_index = 0;
        let mut one_past_max_index = root_node.get_cell_count();

        while min_index != one_past_max_index {
            let index = (min_index + one_past_max_index) / 2;
            let key_at_index = root_node.get_key(index);

            if key_at_index == key {
                return Self::new(table, page_num, index);
            }

            if key_at_index > key {
                one_past_max_index = index;
            } else {
                min_index = index + 1;
            }
        }

        return Self::new(table, page_num, min_index);
    }

    pub fn get_page_num(&self) -> usize {
        self.page_num
    }

    pub fn get_cell_num(&self) -> usize {
        self.cell_num
    }

    pub fn is_end_of_table(&self) -> bool {
        self.end_of_table
    }

    pub fn table_end(table: &'a mut Table) -> Cursor {
        let page_num = table.get_root_page_num();
        let root_node = table.get_pager_mut().get_node_mut(0);
        let cell_count = root_node.get_cell_count();
        Cursor {
            table,
            page_num,
            cell_num: cell_count,
            end_of_table: true,
        }
    }

    pub fn advance(&mut self) {
        self.cell_num += 1;
        if self.cell_num >= self.table
            .get_pager_mut()
            .get_node_mut(self.page_num)
            // -1 account for index vs count
            .get_cell_count()
        {
            self.end_of_table = true;
        }
    }

    pub fn select(&mut self) -> &[u8] {
        self.table.select(self.page_num, self.cell_num)
    }

    //todo: cursor should probably not know about row
    pub fn insert(&mut self, row: &Row) {
        self.table.insert(self.page_num, self.cell_num, row);
    }
}

#[cfg(test)]
mod test {
    use std::{fs, panic};
    use crate::storage::pager::Pager;

    use super::*;

    #[test]
    fn test_cursor() {
        let test_db_path = "test.db";
        let mut table = Table::open_db_connection(test_db_path).unwrap();
        let row = Row::new(1, "test".to_string(), "test".to_string());
        table.insert(0, 0, &row);
        table.flush().unwrap();

        let mut cursor = Cursor::table_start(&mut table);
        assert_eq!(cursor.get_page_num(), 0);
        assert_eq!(cursor.is_end_of_table(), false);
        cursor.advance();
        assert_eq!(cursor.is_end_of_table(), true);

        let mut cursor = Cursor::table_end(&mut table);
        assert_eq!(cursor.get_page_num(), 0);
        assert_eq!(cursor.is_end_of_table(), true);
        cursor.advance();
        assert_eq!(cursor.is_end_of_table(), true);
    }

    #[test]
    fn test_cursor_find() {
        let test_db_path = "test.db";
        let mut table = Table::open_db_connection(test_db_path).unwrap();
        let row = Row::new(1, "test".to_string(), "test".to_string());
        table.insert(0, 0, &row);
        let row = Row::new(2, "test".to_string(), "test".to_string());
        table.insert(0, 1, &row);
        let row = Row::new(3, "test".to_string(), "test".to_string());
        table.insert(0, 2, &row);
        table.flush().unwrap();

        let mut cursor = Cursor::table_find(&mut table, 2);
        assert_eq!(cursor.get_page_num(), 0);
        assert_eq!(cursor.get_cell_num(), 1);
        assert_eq!(cursor.is_end_of_table(), false);
        cursor.advance();
        assert_eq!(cursor.is_end_of_table(), true);

        fs::remove_file(test_db_path).expect("Failed to remove test database file");
    }
}
