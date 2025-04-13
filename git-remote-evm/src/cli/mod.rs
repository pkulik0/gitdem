use log::{debug, info};
#[cfg(test)]
use std::io::BufReader;
#[cfg(test)]
use std::io::Cursor;
use std::io::{BufRead, Write};
use std::str::FromStr;

mod error;

#[cfg(test)]
use crate::core::reference::Keys;
#[cfg(test)]
use crate::core::remote_helper::MockRemoteHelper;
use crate::core::remote_helper::RemoteHelper;
use crate::core::{hash::Hash, reference::Push};
use error::CLIError;

#[derive(Default, PartialEq)]
enum State {
    #[default]
    None,
    ListingFetches(Vec<Hash>),
    ListingPushes(Vec<Push>),
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

    fn do_fetch(&mut self, hashes: Vec<Hash>) -> Result<(), CLIError> {
        info!("fetch: {:?}", hashes);

        for hash in hashes {
            self.remote_helper.fetch(hash)?;
        }

        writeln!(self.stdout)?;
        Ok(())
    }

    fn do_push(&mut self, refs: Vec<Push>) -> Result<(), CLIError> {
        info!("push: {:?}", refs);

        let result = self.remote_helper.push(refs.clone());
        for reference in refs {
            match &result {
                Ok(_) => {
                    writeln!(self.stdout, "ok {}", reference.remote)?;
                }
                Err(e) => {
                    writeln!(self.stdout, "error {} {:?}", reference.remote, e.to_string())?;
                }
            }
        }
        writeln!(self.stdout)?;

        return match result {
            Ok(_) => {
                info!("push complete");
                Ok(())
            },
            Err(e) => Err(e.into()),
        }
    }

    fn handle_line(&mut self, line: String) -> Result<(), CLIError> {
        if line == "\n" {
            match std::mem::take(&mut self.state) {
                State::None => return Err(CLIError::EndOfInput),
                State::ListingFetches(hashes) => return self.do_fetch(hashes),
                State::ListingPushes(refs) => return self.do_push(refs),
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
                let is_for_push = match args.len() {
                    0 => false,
                    1 => match args[0] {
                        "for-push" => true,
                        _ => return Err(CLIError::InvalidArgument(args[0].to_string())),
                    },
                    _ => return Err(CLIError::MalformedLine(line)),
                };

                for reference in self.remote_helper.list(is_for_push)? {
                    response.push_str(&format!("{}\n", reference));
                }
            }
            "fetch" => {
                if args.len() != 2 {
                    return Err(CLIError::MalformedLine(line));
                }

                let hash = Hash::from_str(args[0])
                    .map_err(|_| CLIError::InvalidArgument(args[0].to_string()))?;

                match &mut self.state {
                    State::None => {
                        debug!("new fetch list with: {:?}", hash);
                        self.state = State::ListingFetches(vec![hash]);
                    }
                    State::ListingFetches(hashes) => {
                        debug!("appending fetch to list: {:?}", hash);
                        hashes.push(hash);
                    }
                    State::ListingPushes(_) => return Err(CLIError::IllegalState(line)),
                }
            }
            "push" => {
                if args.len() != 1 {
                    return Err(CLIError::MalformedLine(line));
                }

                let mut arg = args[0];

                let is_force = arg.starts_with("+");
                if is_force {
                    arg = &arg[1..];
                }

                let parts = arg.split(':').collect::<Vec<&str>>();
                if parts.len() != 2 {
                    return Err(CLIError::MalformedLine(line));
                }

                let local = parts[0].to_string();
                let remote = parts[1].to_string();
                let reference = Push::new(local, remote, is_force);

                match &mut self.state {
                    State::None => {
                        debug!("new push list with: {:?}", reference);
                        self.state = State::ListingPushes(vec![reference]);
                    }
                    State::ListingPushes(refs) => {
                        debug!("appending push to list: {:?}", reference);
                        refs.push(reference);
                    }
                    State::ListingFetches(_) => return Err(CLIError::IllegalState(line)),
                }
            }
            _ => return Err(CLIError::UnknownCommand(line)),
        }

        if self.state == State::None {
            writeln!(self.stdout, "{}", response)?;
            if !response.is_empty() {
                info!("{}:\n{}", command, response);
            }
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
                    Ok(_) => {}
                },
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => return Ok(()),
                    _ => return Err(e.into()),
                },
            }
        }
    }
}

#[test]
fn test_capabilities() {
    let mut stdin = BufReader::new(Cursor::new(b"capabilities\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let mut remote_helper = MockRemoteHelper::new();
    remote_helper
        .expect_capabilities()
        .returning(|| vec!["*fetch", "*push"]);
    let mut cli = CLI::new(
        Box::new(remote_helper),
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );

    cli.run().expect("failed to run cli");
    assert_eq!(stdout, b"*fetch\n*push\n\n");
    assert_eq!(stderr, b"");
}

#[test]
fn test_list() {
    // Case 1: No refs
    let mut stdin = BufReader::new(Cursor::new(b"list\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let mut remote_helper = MockRemoteHelper::new();
    remote_helper
        .expect_list()
        .returning(|_is_for_push| Ok(vec![]));
    let mut cli = CLI::new(
        Box::new(remote_helper),
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    cli.run().expect("failed to run cli");
    assert_eq!(stdout, b"\n"); // new line indicates the end of the list
    assert_eq!(stderr, b"");

    // Case 2: Some refs
    let mut stdin = BufReader::new(Cursor::new(b"list\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    use crate::core::reference::Reference;
    let refs = vec![
        Reference::Normal {
            name: "refs/heads/main".to_string(),
            hash: Hash::from_str("4e1243bd22c66e76c2ba9eddc1f91394e57f9f83")
                .expect("failed to create hash"),
        },
        Reference::Symbolic {
            name: "refs/heads/main".to_string(),
            target: "refs/heads/main".to_string(),
        },
        Reference::KeyValue {
            key: Keys::ObjectFormat,
            value: "sha1".to_string(),
        },
    ];

    let refs_clone = refs.clone();
    let mut remote_helper = MockRemoteHelper::new();
    remote_helper
        .expect_list()
        .returning(move |_is_for_push| Ok(refs_clone.clone()));
    let mut cli = CLI::new(
        Box::new(remote_helper),
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    cli.run().expect("failed to run cli");
    assert_eq!(
        stdout,
        format!("{}\n{}\n{}\n\n", refs[0], refs[1], refs[2]).as_bytes()
    );
    assert_eq!(stderr, b"");
}
