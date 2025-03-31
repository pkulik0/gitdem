use crate::remote_helper::RemoteHelperError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CLIError {
    EndOfInput,
    Command(RemoteHelperError),
    UnknownCommand(String),
    InputOutput(std::io::Error),
}

impl Error for CLIError {}

impl std::fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CLIError::EndOfInput => write!(f, "end of input"),
            CLIError::Command(e) => write!(f, "command error: {}", e),
            CLIError::UnknownCommand(command) => write!(f, "unknown command: {:?}", command),
            CLIError::InputOutput(e) => write!(f, "input/output error: {}", e),
        }
    }
}

impl From<std::io::Error> for CLIError {
    fn from(e: std::io::Error) -> Self {
        CLIError::InputOutput(e)
    }
}

impl From<RemoteHelperError> for CLIError {
    fn from(e: RemoteHelperError) -> Self {
        CLIError::Command(e)
    }
}
