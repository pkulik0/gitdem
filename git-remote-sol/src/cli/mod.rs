use crate::remote_helper::RemoteHelper;
use log::{debug, info};
use std::io::{BufRead, Write};

mod error;
#[cfg(test)]
mod tests;

use error::CLIError;

pub struct CLI<'a> {
    remote_helper: Box<dyn RemoteHelper>,
    stdin: &'a mut dyn BufRead,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
    remote_name: String,
    remote_url: String,
}

impl<'a> CLI<'a> {
    pub fn new(
        remote_helper: Box<dyn RemoteHelper>,
        stdin: &'a mut dyn BufRead,
        stdout: &'a mut dyn Write,
        stderr: &'a mut dyn Write,
        remote_name: String,
        remote_url: String,
    ) -> Self {
        info!("remote: {}, url: {}", remote_name, remote_url);
        Self {
            remote_helper,
            stdin,
            stdout,
            stderr,
            remote_name,
            remote_url,
        }
    }

    fn handle_command(&mut self, command: &str) -> Result<(), CLIError> {
        match command {
            "capabilities\n" => {
                debug!("returning capabilities");
                writeln!(
                    self.stdout,
                    "{}\n",
                    self.remote_helper.capabilities().join(",")
                )?;
            }
            "list\n" => {
                debug!("listing refs");
                let refs = self.remote_helper.list()?;
                for reference in refs {
                    writeln!(self.stdout, "{}", reference)?;
                }
                writeln!(self.stdout)?; // needs a new line after the list
            }
            "fetch\n" => {
                debug!("fetching");
            }
            "\n" => {
                return Err(CLIError::EndOfInput);
            }
            _ => {
                return Err(CLIError::UnknownCommand(command.to_string()));
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CLIError> {
        let mut command = String::new();
        loop {
            match self.stdin.read_line(&mut command) {
                Ok(_) => match self.handle_command(command.as_str()) {
                    Ok(_) => {}
                    Err(CLIError::EndOfInput) => return Ok(()),
                    Err(e) => return Err(e),
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => return Ok(()),
                    _ => return Err(e.into()),
                },
            }
            command.clear();
        }
    }
}
