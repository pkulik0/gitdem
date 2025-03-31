use crate::remote_helper::{RemoteHelper, reference::Reference};
use log::{debug, info};
use std::io::{BufRead, Write};

mod error;
#[cfg(test)]
mod tests;

use error::CLIError;

enum State {
    None,
    ListingFetches(Vec<Reference>),
}

pub struct CLI<'a> {
    remote_helper: Box<dyn RemoteHelper>,

    stdin: &'a mut dyn BufRead,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,

    state: State,
}

impl<'a> CLI<'a> {
    pub fn new(
        remote_helper: Box<dyn RemoteHelper>,
        stdin: &'a mut dyn BufRead,
        stdout: &'a mut dyn Write,
        stderr: &'a mut dyn Write,
    ) -> Self {
        Self {
            remote_helper,
            stdin,
            stdout,
            stderr,
            state: State::None,
        }
    }

    fn handle_line(&mut self, line: String) -> Result<(), CLIError> {
        if line == "\n" {
            match &self.state {
                State::None => return Err(CLIError::EndOfInput),
                State::ListingFetches(refs) => {
                    // TODO: Implement
                    info!("fetch: {:?}", refs);
                    self.state = State::None;
                    return Ok(());
                }
            }
        }

        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if parts.len() == 0 {
            return Err(CLIError::MalformedLine(line));
        }

        let command = parts[0];
        let args = parts[1..].to_vec();
        debug!("command: {:?}, args: {:?}", command, args);

        let mut response = String::new();
        match command {
            "capabilities" => {
                if args.len() != 0 {
                    return Err(CLIError::MalformedLine(line));
                }

                response = format!("{}\n", self.remote_helper.capabilities().join("\n"));
            }
            "list" => {
                if args.len() != 0 {
                    return Err(CLIError::MalformedLine(line));
                }

                for reference in self.remote_helper.list()? {
                    response.push_str(&format!("{}\n", reference));
                }
            }
            "fetch" => {
                if args.len() != 2 {
                    return Err(CLIError::MalformedLine(line));
                }

                let hash = args[0].to_string();
                let ref_name = args[1].to_string();
                let reference = Reference::new_with_hash(ref_name, hash);

                match &mut self.state {
                    State::None => {
                        debug!("new fetch list with: {:?}", reference);
                        self.state = State::ListingFetches(vec![reference]);
                    }
                    State::ListingFetches(refs) => {
                        debug!("appending fetch to list: {:?}", reference);
                        refs.push(reference);
                    }
                }
            }
            _ => return Err(CLIError::UnknownCommand(line)),
        }

        match &self.state {
            State::None => {
                writeln!(self.stdout, "{}", response)?;
                if !response.is_empty() {
                    info!("{}:\n{}", command, response);
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CLIError> {
        loop {
            let mut line = String::new();
            match self.stdin.read_line(&mut line) {
                Ok(0) => return Ok(()),
                Ok(_) => match self.handle_line(line) {
                    Err(CLIError::EndOfInput) => return Ok(()),
                    Err(e) => return Err(e),
                    Ok(_) => {},
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => return Ok(()),
                    _ => return Err(e.into()),
                },
            }
        }
    }
}
