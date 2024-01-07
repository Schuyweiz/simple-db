use crate::storage::constant::ROWS_PER_PAGE;
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
        let end_of_table = table.get_page_mut().get_node_mut(0).get_cell_count() == 0;
        Cursor {
            table,
            page_num: 0,
            cell_num: 0,
            end_of_table,
        }
    }

    pub fn get_page_num(&self) -> usize {
        self.page_num
    }

    pub fn is_end_of_table(&self) -> bool {
        self.end_of_table
    }

    pub fn table_end(table: &'a mut Table) -> Cursor {
        let page_num = table.get_root_page_num();
        let root_node = table.get_page_mut().get_node_mut(0);
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
        if self.cell_num
            >= self
                .table
                .get_page_mut()
                .get_node_mut(self.page_num)
                .get_cell_count()
        {
            self.end_of_table = true;
        }
    }

    pub fn select(&mut self) -> &[u8] {
        self.table.select(self.page_num, self.cell_num)
    }

    pub fn insert(&mut self, row: &Row) {
        self.table.insert(self.page_num, self.cell_num, row);
    }
}
