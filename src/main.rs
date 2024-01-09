use cli::cli_parser::CliParser;
use command::meta_cmd_handler::MetaCommand;
use command::meta_cmd_handler::MetaCommandHandler;
use command::sql_cmd_handler::SqlCommand;
use command::sql_cmd_handler::SqlCommandHandler;
use storage::cursor::Cursor;
use storage::row::Row;
use storage::table::Table;

mod cli;
mod command;
mod storage;

fn main() {
    let cli_parser = CliParser::new();
    let file_path = cli_parser.parse_file_path();

    let mut table = Table::open_db_connection(&file_path).unwrap();
    let meta_cmd_handler = MetaCommandHandler::new();
    let sql_cmd_handler = SqlCommandHandler::new();

    loop {
        let user_input = cli_parser.parse_input();

        let meta_cmd = meta_cmd_handler.handle(&user_input);
        if meta_cmd.is_some() {
            match meta_cmd.unwrap() {
                MetaCommand::Exit => {
                    table.flush().unwrap();
                    break;
                }
                MetaCommand::Unknown => {
                    continue;
                }
            }
        }

        let sql_cmd = sql_cmd_handler
            .handle(&user_input)
            .unwrap_or_else(|error| panic!("Failed to parse command due to {:?}", error));

        match sql_cmd {
            SqlCommand::Insert(row) => execute_insert(&mut table, row),
            SqlCommand::Select => execute_select(&mut table),
            SqlCommand::Unknown => {}
        }
    }
}

fn execute_select(table: &mut Table) {
    let mut cursor = Cursor::table_find(table, 0);
    let mut table_start_cursor = cursor.to_table_start();
    while !table_start_cursor.is_end_of_table() {
        let row = Row::deserialize(table_start_cursor.select()).unwrap();
        println!("{:?}", row);
        table_start_cursor.advance();
    }
}

fn execute_insert(table: &mut Table, row: Row) {
    let mut cursor = Cursor::table_find(table, row.get_id() as usize);
    cursor.insert(&row)
}

//todo: usize should be dealt with in a better way
