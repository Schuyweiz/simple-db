use std::io;
use std::io::Write;
use anyhow::Result;

fn main() {
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
            match process_meta_command(user_input_trimmed) {
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

        match prepare_statement(user_input_trimmed) {
            Ok(Statement::Insert) => {
                println!("This is where we would do an insert.");
            }
            Ok(Statement::Select) => {
                println!("This is where we would do a select.");
            }
            Ok(Statement::Unknown) => {
                println!("Unknown command")
            }
            Err(err) => { panic!("Error happened when preparing statement {:?}", err) }
        }
    }
}

//todo: abandon anyhow and use std::error::Error
fn process_meta_command(input: &str) -> MetaCommand {
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
        "insert" => Ok(Statement::Insert),
        "select" => Ok(Statement::Select),
        _ => Ok(Statement::Unknown)
    }
}

enum MetaCommand {
    Exit,
    Unknown,
}

enum Statement {
    Insert,
    Select,
    Unknown,
}