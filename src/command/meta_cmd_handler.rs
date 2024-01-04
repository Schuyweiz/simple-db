pub struct MetaCommandHandler;

impl MetaCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle(&self, command: &str) -> Option<MetaCommand> {
        if !command.starts_with(".") {
            return None;
        }

        match command {
            ".exit" => {
                println!("Exiting...");
                Some(MetaCommand::Exit)
            }
            _ => {
                println!("Unknown command.");
                Some(MetaCommand::Unknown)
            }
        }
    }
}

pub enum MetaCommand {
    Exit,
    Unknown,
}