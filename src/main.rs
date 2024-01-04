use cli::cli_parser::CliParser;
use command::meta_cmd_handler::MetaCommand;
use command::meta_cmd_handler::MetaCommandHandler;
use command::sql_cmd_handler::SqlCommand;
use command::sql_cmd_handler::SqlCommandHandler;
use storage::row::Row;
use storage::table::Table;

mod storage;
mod cli;
mod command;

fn main() {
    let mut table = Table::new();
    let parser = CliParser::new();
    let meta_cmd_handler = MetaCommandHandler::new();
    let sql_cmd_handler = SqlCommandHandler::new();

    loop {
        let user_input = parser.parse_input();

        let meta_cmd = meta_cmd_handler.handle(&user_input);
        if meta_cmd.is_some() {
            match meta_cmd.unwrap() {
                MetaCommand::Exit => { break; }
                MetaCommand::Unknown => { continue; }
            }
        }

        let sql_cmd = sql_cmd_handler.handle(&user_input);

        let statement = sql_cmd.unwrap_or_else(
            |error| { panic!("Failed to prepare a statement due to {:?}", error) }
        );
        match statement {
            SqlCommand::Insert(row) => { table.insert(&row.serialize().unwrap()); }
            SqlCommand::Select => { execute_select(&table) }
            SqlCommand::Unknown => {}
        }
    }
}

fn execute_select(table: &Table) {
    let current_row_count = table.get_current_row_count();
    for i in 0..current_row_count {
        let data_opt = table.select(i);
        if data_opt.is_none() {
            continue;
        } else {
            println!("{:?}", Row::deserialize(data_opt.unwrap()).unwrap());
        }
    }
}
