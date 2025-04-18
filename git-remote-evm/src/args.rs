use regex::Regex;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const EVM_ADDRESS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^0x[a-fA-F0-9]{40}$").expect("failed to create evm address regex")
});
const INVALID_REF_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^\.)|(^/)|(\.\.)|([:?\[\\^~\s*])|(\.lock$)|(/$)|(@\{)|([\x00-\x1f])")
        .expect("failed to create invalid ref name regex")
});

const EXECUTABLE_PREFIX: &str = "git-remote-";

#[derive(Debug, Clone, PartialEq)]
pub struct ArgsError {
    what: String,
    value: String,
}

impl Error for ArgsError {}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid {}: {}", self.what, self.value)
    }
}

#[derive(Debug)]
pub struct Args {
    protocol: String,
    directory: PathBuf,
    remote_name: Option<String>,
    address: Option<[u8; 20]>,
}

impl Args {
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub fn remote_name(&self) -> Option<&str> {
        self.remote_name.as_deref()
    }

    pub fn address(&self) -> Option<&[u8; 20]> {
        self.address.as_ref()
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub fn parse(args: &[String], git_dir: PathBuf) -> Result<Self, ArgsError> {
        let protocol = protocol_from_arg(&args[0])?;
        match args.len() {
            2 => {
                let remote_name = args[1].clone();
                return Ok(Self {
                    protocol: protocol.to_string(),
                    directory: git_dir,
                    remote_name: Some(remote_name),
                    address: None, // Needs to be read from the saved remote
                });
            }
            3 => {
                let address_str = address_from_arg(&args[2], &protocol)?;
                let address_str = address_str.strip_prefix("0x").ok_or(ArgsError {
                    what: "address".to_string(),
                    value: address_str.to_string(),
                })?;
                let address = hex::decode(address_str).map_err(|e| ArgsError {
                    what: "address".to_string(),
                    value: e.to_string(),
                })?;
                let address: [u8; 20] = *address.as_array().ok_or(ArgsError {
                    what: "address".to_string(),
                    value: "invalid address".to_string(),
                })?;

                let remote_name = if args[1] == args[2] {
                    None
                } else {
                    let remote_name = args[1].clone();
                    if !validate_remote_name(&remote_name) {
                        return Err(ArgsError {
                            what: "remote name".to_string(),
                            value: args[1].clone(),
                        });
                    }
                    Some(remote_name)
                };

                Ok(Self {
                    protocol: protocol.to_string(),
                    directory: git_dir,
                    remote_name,
                    address: Some(address),
                })
            }
            _ => {
                return Err(ArgsError {
                    what: "argument count".to_string(),
                    value: args.len().to_string(),
                });
            }
        }
    }
}

fn address_from_arg<'a>(arg: &'a str, protocol: &str) -> Result<&'a str, ArgsError> {
    let address_prefix = format!("{}://", protocol);
    let address = match arg.find(&address_prefix) {
        Some(start) => &arg[start + address_prefix.len()..],
        None => arg,
    };
    match validate_address(address) {
        false => Err(ArgsError {
            what: "address".to_string(),
            value: arg.to_string(),
        }),
        true => Ok(address),
    }
}

#[test]
fn test_address_from_arg() {
    let address_str = "0xc0ffee254729296a45a3885639AC7E10F9d54979";
    let protocol = "eth";
    let prefixed = format!("{}://{}", protocol, address_str);

    let address = address_from_arg(&prefixed, protocol).expect("failed to get address");
    assert_eq!(address, address_str);

    let address = address_from_arg(address_str, protocol).expect("failed to get address");
    assert_eq!(address, address_str);

    let invalid_address = "invalid _";
    let address = address_from_arg(invalid_address, protocol).expect_err("expected error");
    assert_eq!(
        address,
        ArgsError {
            what: "address".to_string(),
            value: invalid_address.to_string(),
        }
    );
}

fn protocol_from_arg(arg: &str) -> Result<&str, ArgsError> {
    let err = ArgsError {
        what: "protocol".to_string(),
        value: arg.to_string(),
    };

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
    let protocol = protocol_from_arg("git-remote-eth").expect("failed to get protocol");
    assert_eq!(protocol, "eth");

    let protocol = protocol_from_arg("git-remote-arb1").expect("failed to get protocol");
    assert_eq!(protocol, "arb1");

    let protocol = protocol_from_arg("git-remote-base").expect("failed to get protocol");
    assert_eq!(protocol, "base");

    let protocol = protocol_from_arg("/some/path/git-remote-eth").expect("failed to get protocol");
    assert_eq!(protocol, "eth");

    let protocol = protocol_from_arg("/projects/git-remote-evm/build/git-remote-evm")
        .expect("failed to get protocol");
    assert_eq!(protocol, "evm");

    let protocol = protocol_from_arg("git-remote-").expect_err("expected error");
    assert_eq!(
        protocol,
        ArgsError {
            what: "protocol".to_string(),
            value: "git-remote-".to_string(),
        }
    );

    let protocol = protocol_from_arg("\\").expect_err("expected error");
    assert_eq!(
        protocol,
        ArgsError {
            what: "protocol".to_string(),
            value: "\\".to_string(),
        }
    );
}

fn validate_remote_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    !INVALID_REF_NAME_REGEX.is_match(name)
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
        "with\nnewline", // Invalid (control character \n)
    ];
    for remote_name in invalid_names {
        let result = validate_remote_name(remote_name);
        if result {
            panic!("expected invalid remote name: {}", remote_name);
        }
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
        let result = validate_remote_name(remote_name);
        if !result {
            panic!("expected valid remote name: {}", remote_name);
        }
    }
}

fn validate_address(address: &str) -> bool {
    EVM_ADDRESS_REGEX.is_match(address)
}

#[test]
fn test_validate_address() {
    // Successes

    let address = "0xc0ffee254729296a45a3885639AC7E10F9d54979";
    assert!(validate_address(address));

    let address = "0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97";
    assert!(validate_address(address));

    let address = "0x388C818CA8B9251b393131C08a736A67ccB19297";
    assert!(validate_address(address));

    let address = "0xC6093Fd9cc143F9f058938868b2df2daF9A91d28";
    assert!(validate_address(address));

    // Failures

    let address = "0xc0ffee254729296a45a3885639AC7E10F9d54979!";
    assert!(!validate_address(address));

    let address = "0xC6093Fd9cc143F9";
    assert!(!validate_address(address));

    let address = "0x388C818CA8B9251b393131C08a736A67ccB192972";
    assert!(!validate_address(address));

    let address = "";
    assert!(!validate_address(address));

    let address = "0x 123";
    assert!(!validate_address(address));
}

#[test]
fn test_parse() {
    let git_dir = PathBuf::from("/some-dir");

    // Case 1: argc == 2
    let executable = "git-remote-eth";
    let remote_name = "test-remote";
    let cmd_args = vec![executable.to_string(), remote_name.to_string()];
    let args = Args::parse(&cmd_args, git_dir.clone()).expect("failed to parse args");
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), Some(remote_name));
    assert_eq!(args.address(), None);

    // Case 2: argc == 3, argv[1] != argv[2]
    let remote_name = "test-remote";
    let address = "eth://0xc0ffee254729296a45a3885639AC7E10F9d54979";
    let address_no_prefix = address
        .strip_prefix("eth://0x")
        .expect("failed to strip prefix from address");
    let cmd_args = vec![
        executable.to_string(),
        remote_name.to_string(),
        address.to_string(),
    ];
    let args = Args::parse(&cmd_args, git_dir.clone()).expect("failed to parse args");
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), Some(remote_name));
    assert_eq!(
        hex::encode(args.address().expect("failed to get address")).to_lowercase(),
        address_no_prefix.to_lowercase()
    );

    // Case 3: argc == 3, argv[1] == argv[2]
    let cmd_args = vec![
        executable.to_string(),
        address.to_string(),
        address.to_string(),
    ];
    let args = Args::parse(&cmd_args, git_dir.clone()).expect("failed to parse args");
    assert_eq!(
        args.directory().display().to_string(),
        git_dir.display().to_string()
    );
    assert_eq!(args.remote_name(), None);
    assert_eq!(
        hex::encode(args.address().expect("failed to get address")).to_lowercase(),
        address_no_prefix.to_lowercase()
    );

    // Case 4: argc < 2
    let cmd_args = vec![executable.to_string()];
    let err = Args::parse(&cmd_args, git_dir.clone()).expect_err("expected error");
    assert_eq!(
        err,
        ArgsError {
            what: "argument count".to_string(),
            value: "1".to_string(),
        }
    );
}
