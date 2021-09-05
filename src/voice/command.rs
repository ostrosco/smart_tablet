use crate::settings::Language;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, time::Duration};

/// An enumeration of all possible command types that the system knows how to execute from voice
/// commands. This is definitely a non-exhausive list.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum CommandType {
    Weather,
    News,
    Timer,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Weather,
    News,
    Timer(Duration),
}

pub struct CommandParser {
    command_map: HashMap<String, CommandType>,
}

impl CommandParser {
    pub fn init(language: Language) -> Result<Self, Box<dyn std::error::Error>> {
        let command_suffix = match language {
            Language::English => "en",
        };

        let command_file_path: String = format!("commands_{}.json", command_suffix);
        let mut command_file = File::open(command_file_path)?;
        let mut data = String::new();
        command_file.read_to_string(&mut data)?;
        let command_map: HashMap<String, CommandType> = serde_json::from_str(&data)?;
        Ok(Self { command_map })
    }

    pub fn parse(&self, command: &str) -> Option<Command> {
        // We're going to do a pretty naive lookup here. We're going to scan the received command
        // for words that help us indicate what type of command this may be. Based on whether any
        // of the words that we map to a command are present, we'll then handle the command
        // specifically.
        let command_type = command
            .split_whitespace()
            .find_map(|cmd| self.command_map.get(cmd));

        if let Some(command) = command_type {
            match command {
                CommandType::Weather => Some(Command::Weather),
                CommandType::News => Some(Command::News),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let language = Language::English;
        let command_parser = CommandParser::init(language).expect("No command.json file found");
        assert_eq!(
            command_parser.parse("what is the weather"),
            Some(Command::Weather)
        );
        assert_eq!(command_parser.parse("weather"), Some(Command::Weather));
        assert_eq!(command_parser.parse("news"), Some(Command::News));

        // In this case, we have multiple commands as once. Our naive parsing should just pick up
        // the first one.
        assert_eq!(command_parser.parse("weather news"), Some(Command::Weather));
    }
}
