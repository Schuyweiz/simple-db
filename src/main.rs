use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("db > ");
        io::stdout().flush().expect("Failed to flush stdout");

        // creating an empty string object in the heap. Can be mutated to store user input.
        let mut user_input = String::new();
        // takes user input as a mutable reference to mutate it - write user input.
        io::stdin().read_line(&mut user_input).expect("Failed to read line");

        match user_input.trim() {
            ".exit" => {
                println!("Exiting...");
                break;
            }
            _ => { println!("Unknown command.") }
        }
    }
}