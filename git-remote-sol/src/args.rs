use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

const SOLANA_ADDRESS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").expect("failed to create solana address regex")
});

const EXECUTABLE_PREFIX: &str = "git-remote-";

#[derive(Debug, Clone, PartialEq)]
pub enum ArgsError {
    ArgCount(usize, Vec<usize>),
    InvalidAddress(String),
    InvalidProtocol(String),
    InvalidRemoteName(String),
}

impl Error for ArgsError {}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArgCount(count, expected) => {
                write!(
                    f,
                    "unexpected number of arguments: {} (allowed: {:?})",
                    count, expected
                )
            }
            Self::InvalidAddress(address) => write!(f, "invalid address: {:?}", address),
            Self::InvalidProtocol(protocol) => write!(f, "invalid protocol: {:?}", protocol),
            Self::InvalidRemoteName(remote_name) => {
                write!(f, "invalid remote name: {:?}", remote_name)
            }
        }
    }
}

#[derive(Debug)]
pub struct Args {
    remote_name: Option<String>,
    address: Option<String>,
    directory: PathBuf,
}

fn address_from_arg<'a>(arg: &'a str, protocol: &str) -> Result<&'a str, ArgsError> {
    let address_prefix = format!("{}://", protocol);
    let address = match arg.find(&address_prefix) {
        Some(start) => &arg[start + address_prefix.len()..],
        None => arg,
    };
    match validate_address(address) {
        false => Err(ArgsError::InvalidAddress(arg.to_string())),
        true => Ok(address),
    }
}

#[test]
fn test_address_from_arg() {
    let address = address_from_arg("sol://DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ", "sol")
        .expect("failed to get address");
    assert_eq!(address, "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ");

    let address = address_from_arg("DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ", "sol")
        .expect("failed to get address");
    assert_eq!(address, "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ");

    let address = address_from_arg("invalid", "sol").expect_err("expected error");
    assert_eq!(address, ArgsError::InvalidAddress("invalid".to_string()));
}

fn protocol_from_arg(arg: &str) -> Result<&str, ArgsError> {
    let err = ArgsError::InvalidProtocol(arg.to_string());

    let path = Path::new(arg);
    let last_component = path.components().last().ok_or(err.clone())?;

    let executable_name = last_component.as_os_str().to_str().ok_or(err.clone())?;
    if !executable_name.starts_with(EXECUTABLE_PREFIX) {
        return Err(err.clone());
    }

    let protocol = &executable_name[EXECUTABLE_PREFIX.len()..];
    if protocol.is_empty() {
        return Err(err);
    }
    Ok(protocol)
}

#[test]
fn test_protocol_from_arg() {
    let protocol = protocol_from_arg("git-remote-sol").expect("failed to get protocol");
    assert_eq!(protocol, "sol");

    let protocol = protocol_from_arg("/some/path/git-remote-sol").expect("failed to get protocol");
    assert_eq!(protocol, "sol");

    let protocol = protocol_from_arg("/projects/git-remote-sol/build/git-remote-sol")
        .expect("failed to get protocol");
    assert_eq!(protocol, "sol");

    let protocol = protocol_from_arg("git-remote-").expect_err("expected error");
    assert_eq!(
        protocol,
        ArgsError::InvalidProtocol("git-remote-".to_string())
    );

    let protocol = protocol_from_arg("\\").expect_err("expected error");
    assert_eq!(protocol, ArgsError::InvalidProtocol("\\".to_string()));
}

fn validate_remote_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.starts_with("/") || name.ends_with("/") {
        return false;
    }
    if name.ends_with(".lock") {
        return false;
    }
    if name.contains("@{") || name == "@" {
        return false;
    }
    if name.contains("..") {
        return false;
    }
    name.bytes().all(|b| {
        match b {
            // Disallowed ASCII Control Characters (0x00 - 0x1F) and DEL (0x7F)
            0x00..=0x1F | 0x7F => false,
            // Disallowed Symbols: space ~ ^ : ? * [ \
            b' ' | b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\' => false,
            _ => true,
        }
    })
}

#[test]
fn test_validate_remote_name() {
    let invalid_names = vec![
        "",              // Invalid (empty)
        " ",             // Invalid (space)
        "~remote",       // Invalid (tilde)
        "my^remote",     // Invalid (caret)
        "my:remote",     // Invalid (colon)
        "my?remote",     // Invalid (question mark)
        "my*remote",     // Invalid (asterisk)
        "my[remote",     // Invalid (open bracket)
        "my\\remote",    // Invalid (backslash)
        "my remote",     // Invalid (space)
        "remote..two",   // Invalid (contains ..)
        "../remote",     // Invalid (contains ..)
        "remote/",       // Invalid (ends with '/')
        "/remote",       // Invalid (starts with '/')
        "remote.lock",   // Invalid (ends with .lock)
        "remote@{abc}",  // Invalid (contains @{)
        "@",             // Invalid (single @)
        "with\nnewline", // Invalid (control character \n)
    ];
    for remote_name in invalid_names {
        assert!(!validate_remote_name(remote_name));
    }

    let valid_names = vec![
        "origin",
        "upstream",
        "my-remote",
        "remote_123",
        "a.b.c",
        "feature/branch-remote",
        "你好",
    ];
    for remote_name in valid_names {
        assert!(validate_remote_name(remote_name));
    }
}

fn validate_address(address: &str) -> bool {
    SOLANA_ADDRESS_REGEX.is_match(address)
}

#[test]
fn test_validate_address() {
    let address = "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ";
    assert!(validate_address(address));

    let too_short = "DBWrGX82Abj1R9Hx";
    assert!(!validate_address(too_short));

    let too_long = "DBWrGX82Abj1R9HxarNuucwSDBWrGX82Abj1R9HxarNuucwSDBWrGX82Abj1R9HxarNuucwS";
    assert!(!validate_address(too_long));

    let invalid_chars = "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ!";
    assert!(!validate_address(invalid_chars));
}

impl Args {
    pub fn remote_name(&self) -> Option<&str> {
        self.remote_name.as_deref()
    }

    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub fn parse(args: &[String], git_dir: PathBuf) -> Result<Self, ArgsError> {
        match args.len() {
            2 => {
                let remote_name = args[1].clone();
                return Ok(Self {
                    remote_name: Some(remote_name),
                    address: None,
                    directory: git_dir,
                });
            }
            3 => {
                let protocol = protocol_from_arg(&args[0])?;
                let address = address_from_arg(&args[2], &protocol)?;

                let remote_name = if args[1] == args[2] {
                    None
                } else {
                    let remote_name = args[1].clone();
                    if !validate_remote_name(&remote_name) {
                        return Err(ArgsError::InvalidRemoteName(args[1].clone()));
                    }
                    Some(remote_name)
                };

                Ok(Self {
                    remote_name,
                    address: Some(address.to_string()),
                    directory: git_dir,
                })
            }
            _ => return Err(ArgsError::ArgCount(args.len(), vec![2, 3])),
        }
    }
}

#[test]
fn test_parse() {
    let git_dir = PathBuf::from("/some-dir");

    // Case 1: argc == 2
    let executable = "git-remote-sol";
    let remote_name = "test-remote";
    let cmd_args = vec![executable.to_string(), remote_name.to_string()];
    let args = Args::parse(&cmd_args, git_dir.clone()).unwrap();
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), Some(remote_name));
    assert_eq!(args.address(), None);

    // Case 2: argc == 3, argv[1] != argv[2]
    let remote_name = "test-remote";
    let address = "sol://DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ";
    let address_no_prefix = address
        .strip_prefix("sol://")
        .expect("failed to strip prefix from address");
    let cmd_args = vec![
        executable.to_string(),
        remote_name.to_string(),
        address.to_string(),
    ];
    let args = Args::parse(&cmd_args, git_dir.clone()).unwrap();
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), Some(remote_name));
    assert_eq!(args.address(), Some(address_no_prefix));

    // Case 3: argc == 3, argv[1] == argv[2]
    let cmd_args = vec![
        executable.to_string(),
        address.to_string(),
        address.to_string(),
    ];
    let args = Args::parse(&cmd_args, git_dir.clone()).unwrap();
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), None);
    assert_eq!(args.address(), Some(address_no_prefix));

    // Case 4: argc < 2
    let cmd_args = vec![executable.to_string()];
    let err = Args::parse(&cmd_args, git_dir.clone()).expect_err("expected error");
    assert_eq!(err, ArgsError::ArgCount(1, vec![2, 3]));
}
