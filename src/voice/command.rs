use super::number::parse_number_from_voice;
use crate::settings::Language;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryInto, fs::File, io::Read, time::Duration};

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

        println!("Command type: {:?}", command_type);
        if let Some(cmd_type) = command_type {
            match cmd_type {
                CommandType::Weather => Some(Command::Weather),
                CommandType::News => Some(Command::News),
                CommandType::Timer => self.parse_timer(command),
            }
        } else {
            None
        }
    }

    // Again, this is going to be English centric for now until we figure out a sane way to keep
    // the effective mapping between words and what they mean.
    fn parse_timer(&self, command: &str) -> Option<Command> {
        if let Some(number) = parse_number_from_voice(command) {
            let number: u64 = number.try_into().unwrap();
            if command.contains("second") {
                Some(Command::Timer(Duration::from_secs(number)))
            } else if command.contains("minute") {
                Some(Command::Timer(Duration::from_secs(number * 60)))
            } else if command.contains("hour") {
                Some(Command::Timer(Duration::from_secs(number * 60 * 60)))
            } else {
                None
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

    #[test]
    fn test_timer() {
        let language = Language::English;
        let command_parser = CommandParser::init(language).expect("No command.json file found");
        assert_eq!(
            command_parser.parse("set timer for thirty hours"),
            Some(Command::Timer(Duration::from_secs(30 * 60 * 60)))
        );
        assert_eq!(
            command_parser.parse("set timer for fourty two minutes"),
            Some(Command::Timer(Duration::from_secs(42 * 60)))
        );
        assert_eq!(
            command_parser
                .parse("set timer for one hundred and twelve thousand and sixty two seconds"),
            Some(Command::Timer(Duration::from_secs(112_062)))
        );
    }
}
