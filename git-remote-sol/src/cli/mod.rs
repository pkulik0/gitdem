use crate::remote_helper::RemoteHelper;
use std::error::Error;
use std::io::{BufRead, Write};
use log::{debug, error, info};
#[cfg(test)]
mod tests;

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

    fn handle_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        match command {
            "capabilities" => {
                debug!("returning capabilities");
                writeln!(
                    self.stdout,
                    "{}",
                    self.remote_helper.capabilities().join(",")
                )?;
            }
            "fetch" => {
                debug!("fetching");
                writeln!(self.stderr, "failed to fetch\n")?;
                std::process::exit(1);
            }
            _ => {
                error!("Unknown command: {}", command);
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
