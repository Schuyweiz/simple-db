pub struct SqlCommandHandler;

use anyhow::Result;
//todo: remove when more general approach is adapted
use crate::storage::row::Row;

impl SqlCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle(&self, command: &str) -> Result<SqlCommand> {
        let mut tokens = command.split_whitespace();
        let command = tokens.next().unwrap();
        let args = tokens.collect::<Vec<&str>>();

        match command {
            "insert" => {
                let id = args[0].parse::<u32>().unwrap();
                let user_name = args[1].to_string();
                let email = args[2].to_string();

                Ok(SqlCommand::Insert(Row::new(id, user_name, email)))
            }
            "select" => Ok(SqlCommand::Select),
            _ => {
                println!("Unknown command.");
                Ok(SqlCommand::Unknown)
            }
        }
    }
}

pub enum SqlCommand {
    Insert(Row),
    Select,
    Unknown,
}
