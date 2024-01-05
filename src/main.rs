use cli::cli_parser::CliParser;
use command::meta_cmd_handler::MetaCommand;
use command::meta_cmd_handler::MetaCommandHandler;
use command::sql_cmd_handler::SqlCommand;
use command::sql_cmd_handler::SqlCommandHandler;
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
            SqlCommand::Insert(row) => {
                table.insert(&row.serialize().unwrap());
            }
            SqlCommand::Select => execute_select(&mut table),
            SqlCommand::Unknown => {}
        }
    }
}

fn execute_select(table: &mut Table) {
    let current_row_count = table.get_current_row_count();
    for i in 0..current_row_count {
        let data_opt = table.select(i);
        println!("{:?}", Row::deserialize(data_opt).unwrap());
    }
}
