mod table;

use table::Row;

use std::io;
use std::io::Write;
use std::str::FromStr;
use anyhow::Result;
use crate::table::Table;

fn main() {
    let mut table = Table::new();
    loop {
        print!("db > ");
        io::stdout().flush().expect("Failed to flush stdout");

        // creating an empty string object in the heap. Can be mutated to store user input.
        let mut user_input = String::new();
        // takes user input as a mutable reference to mutate it - write user input.
        io::stdin().read_line(&mut user_input).expect("Failed to read line");

        // we dont need the command to be mutable for further operations.
        // so we can user trim() to get a slice of a string and use it from now on.
        // also allows to stop thinking if the string's been trimmed
        let user_input_trimmed = user_input.trim();

        if user_input_trimmed.starts_with(".") {
            match prepare_meta_command(user_input_trimmed) {
                MetaCommand::Exit => {
                    println!("Exiting...");
                    break;
                }
                MetaCommand::Unknown => {
                    println!("Unknown command.");
                    continue;
                }
            }
        }

        let statement = prepare_statement(user_input_trimmed).unwrap_or_else(
            |error| { panic!("Failed to prepare a statement due to {:?}", error) }
        );
        match statement {
            Statement::Insert(row) => {
                let current_row_count = table.get_current_row_count();
                let mut page = table.get_page_mut(current_row_count);
                page.write_slot(current_row_count, &row.serialize().unwrap());
                table.increment_current_row_count()
            }
            Statement::Select => {
                let current_row_count = table.get_current_row_count();
                for i in 0..current_row_count {
                    let page = table.get_page_ref(i);
                    if page.is_none() {
                        continue;
                    } else {
                        let page_slot = page.unwrap().read_slot(i);

                        println!("{:?}", Row::deserialize(&page_slot).unwrap());
                        dbg!(page_slot.as_ptr());
                    }
                }
            }
            Statement::Unknown => {}
        }
    }
}

//todo: abandon anyhow and use std::error::Error
fn prepare_meta_command(input: &str) -> MetaCommand {
    match input.trim() {
        ".exit" => MetaCommand::Exit,
        _ => MetaCommand::Unknown
    }
}

//todo: abandon anyhow and use std::error::Error
fn prepare_statement(input: &str) -> Result<Statement> {
    let statement_args: Vec<&str> = input.split_whitespace().collect();
    let statement_identifier = statement_args[0];

    match statement_identifier {
        "insert" => {
            if statement_args.len() != 4 {
                //todo: replace with an actual error handling later on
                return Err(anyhow::Error::msg("Args size does not match row insert structure."));
            }
            let id = u32::from_str(statement_args[1]).expect("Failed to parse id.");
            let row = Row::new(
                id,
                statement_args[2].into(),
                statement_args[3].into(),
            );

            Ok(Statement::Insert(row))
        }
        "select" => Ok(Statement::Select),
        _ => Ok(Statement::Unknown)
    }
}

enum MetaCommand {
    Exit,
    Unknown,
}

enum Statement {
    Insert(Row),
    Select,
    Unknown,
}