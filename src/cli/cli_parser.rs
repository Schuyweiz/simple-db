use std::io;
use std::io::Write;

pub struct CliParser;

impl CliParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_input(&self) -> String {
        print!("db > ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut buffer = String::new();

        // creating an empty string object in the heap. Can be mutated to store user input.
        // takes user input as a mutable reference to mutate it - write user input.
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");

        // we dont need the command to be mutable for further operations.
        // so we can user trim() to get a slice of a string and use it from now on.
        // also allows to stop thinking if the string's been trimmed
        buffer.trim().to_string()
    }

    pub fn parse_file_path(&self) -> String {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            panic!("Please provide a database file path");
        }

        args[1].clone()
    }
}
