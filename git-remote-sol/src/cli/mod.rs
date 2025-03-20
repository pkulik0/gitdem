use crate::remote_helper::RemoteHelper;
use std::error::Error;
use std::io::{BufRead, Write};

#[cfg(test)]
mod tests;

pub struct CLI<'a> {
    remote_helper: Box<dyn RemoteHelper>,
    stdin: &'a mut dyn BufRead,
    stdout: &'a mut dyn Write,
    stderr: &'a mut dyn Write,
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
        }
    }

    fn handle_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        match command {
            "capabilities" => {
                writeln!(
                    self.stdout,
                    "{}",
                    self.remote_helper.capabilities().join(",")
                )?;
            }
            _ => {
                return Err(format!("Unknown command: {}", command).into());
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut command = String::new();
        loop {
            match self.stdin.read_line(&mut command) {
                Ok(0) => {
                    return Ok(());
                }
                Ok(_) => {
                    self.handle_command(command.trim())?;
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        return Ok(());
                    }
                    _ => {
                        return Err(format!("Error reading command: {}", e).into());
                    }
                },
            }
            command.clear();
        }
    }
}
