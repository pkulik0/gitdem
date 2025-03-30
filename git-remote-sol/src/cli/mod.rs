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
                    "{}\n",
                    self.remote_helper.capabilities().join(",")
                )?;
            }
            "list" => {
                debug!("listing refs");
                let refs = self.remote_helper.list()?;
                for reference in refs {
                    writeln!(self.stdout, "{}", reference)?;
                }
                writeln!(self.stdout)?; // needs a new line after the list
            }
            "fetch" => {
                debug!("fetching");
            }
            _ => {
                return Err(format!("unknown command: \"{}\"", command).into());
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut command = String::new();
        loop {
            match self.stdin.read_line(&mut command) {
                Ok(_) => match command.as_str() {
                    "\n" => return Ok(()), // used by git to signal end of input
                    _ => self.handle_command(command.trim())?,
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => return Ok(()),
                    _ => return Err(format!("failed to read command: {}", e).into())
                },
            }
            command.clear();
        }
    }
}
