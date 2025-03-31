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

    fn handle_line(&mut self, line: String) -> Result<(), CLIError> {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let command = parts[0].trim();
        let args = parts[1..].to_vec();

        match command {
            "capabilities" => {
                if args.len() != 0 {
                    return Err(CLIError::MalformedLine(line));
                }

                let response = format!("{}\n", self.remote_helper.capabilities().join(","));
                writeln!(self.stdout, "{}", response)?;
                debug!("wrote capabilities: {}", response.trim());
            }
            "list" => {
                if args.len() != 0 {
                    return Err(CLIError::MalformedLine(line));
                }

                let mut response = String::new();
                for reference in self.remote_helper.list()? {
                    response.push_str(&format!("{}\n", reference));
                }
                writeln!(self.stdout, "{}", response)?;
                debug!("wrote list: {}", response.trim());
            }
            _ => {
                return Err(CLIError::UnknownCommand(command.to_string()));
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CLIError> {
        loop {
            let mut line = String::new();
            match self.stdin.read_line(&mut line) {
                Ok(_) => match line.as_str() {
                    "\n" => return Ok(()),
                    _ => self.handle_line(line)?,
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => return Ok(()),
                    _ => return Err(e.into()),
                },
            }
        }
    }
}
