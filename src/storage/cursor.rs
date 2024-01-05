use crate::storage::constant::ROWS_PER_PAGE;
use crate::storage::table::Table;

pub struct Cursor<'a> {
    table: &'a mut Table,
    pub row_num: usize,
    end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn table_start(table: &'a mut Table) -> Cursor {
        let end_of_table = table.get_current_row_count() == 0;
        Cursor {
            table,
            row_num: 0,
            end_of_table,
        }
    }

    pub fn is_end_of_table(&self) -> bool {
        self.end_of_table
    }

    pub fn table_end(table: &'a mut Table) -> Cursor {
        let row_num = table.get_current_row_count();
        Cursor {
            table,
            row_num,
            end_of_table: true,
        }
    }

    pub fn cursor_value(&mut self) -> &mut [u8] {
        let row = self.row_num;
        let page_num = row / ROWS_PER_PAGE;

        let page = self.table.get_page_mut().get_page_mut(page_num);
        page.get_slot(row)
    }

    pub fn advance(&mut self) {
        self.row_num += 1;
        self.end_of_table = self.row_num == self.table.get_current_row_count();
    }
}
