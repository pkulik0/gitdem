use super::remote_helper::error::RemoteHelperError;
use crate::core::hash::Hash;
use crate::core::object::{Object, ObjectKind};
use log::{debug, trace};
use mockall::automock;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for GitVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[automock]
pub trait Git {
    fn version(&self) -> Result<GitVersion, RemoteHelperError>;
    fn is_sha256(&self) -> Result<bool, RemoteHelperError>;
    fn resolve_reference(&self, name: &str) -> Result<Hash, RemoteHelperError>;
    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError>;
    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError>;
    fn list_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError>;
    fn list_objects(&self, hash: Hash) -> Result<Vec<Hash>, RemoteHelperError>;
    fn get_address(&self, protocol: &str, remote_name: &str)
    -> Result<[u8; 20], RemoteHelperError>;
}

pub struct SystemGit {
    path: PathBuf,
}

impl SystemGit {
    pub fn new(path: PathBuf) -> Self {
        debug!("git commands will run in: {}", path.to_string_lossy());
        Self { path }
    }
}

impl SystemGit {
    fn rev_list(&self, name: &str) -> Result<Vec<Hash>, RemoteHelperError> {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["rev-list", "--objects", name])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "running git rev-list".to_string(),
                details: Some(e.to_string()),
            })?;
        if !output.status.success() {
            return Err(RemoteHelperError::Failure {
                action: "running git rev-list".to_string(),
                details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git rev-list".to_string(),
            details: Some(e.to_string()),
        })?;

        let mut hashes = vec![];
        for line in stdout.lines() {
            let hash_str = line
                .split_whitespace()
                .next()
                .ok_or(RemoteHelperError::Failure {
                    action: "getting hash from line".to_string(),
                    details: Some(line.to_string()),
                })?;
            let hash = Hash::from_str(hash_str).map_err(|e| RemoteHelperError::Failure {
                action: "parsing hash".to_string(),
                details: Some(e.to_string()),
            })?;
            hashes.push(hash);
        }
        Ok(hashes)
    }
}

impl Git for SystemGit {
    fn version(&self) -> Result<GitVersion, RemoteHelperError> {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["--version"])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting git version".to_string(),
                details: Some(e.to_string()),
            })?;
        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git --version".to_string(),
            details: Some(e.to_string()),
        })?;
        let version =
            stdout
                .trim()
                .strip_prefix("git version ")
                .ok_or(RemoteHelperError::Failure {
                    action: "parsing git version".to_string(),
                    details: Some(stdout.to_string()),
                })?;

        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(RemoteHelperError::Failure {
                action: "parsing git version".to_string(),
                details: Some(version.to_string()),
            });
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|e| RemoteHelperError::Failure {
                action: "parsing git major version".to_string(),
                details: Some(e.to_string()),
            })?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|e| RemoteHelperError::Failure {
                action: "parsing git minor version".to_string(),
                details: Some(e.to_string()),
            })?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|e| RemoteHelperError::Failure {
                action: "parsing git patch version".to_string(),
                details: Some(e.to_string()),
            })?;

        let version = GitVersion {
            major,
            minor,
            patch,
        };
        trace!("retrieved git version: {}", version);
        Ok(version)
    }

    fn is_sha256(&self) -> Result<bool, RemoteHelperError> {
        trace!(
            "checking if git is using sha256 in {}",
            self.path.to_string_lossy()
        );
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["rev-parse", "--show-object-format"])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting git config".to_string(),
                details: Some(e.to_string()),
            })?;
        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git config".to_string(),
            details: Some(e.to_string()),
        })?;
        let is_sha256 = match stdout.trim() {
            "sha256" => true,
            "sha1" => false,
            _ => {
                return Err(RemoteHelperError::Invalid {
                    what: "git object format".to_string(),
                    value: stdout.to_string(),
                });
            }
        };
        debug!("git is using sha256: {}", is_sha256);
        Ok(is_sha256)
    }

    fn get_address(
        &self,
        protocol: &str,
        remote_name: &str,
    ) -> Result<[u8; 20], RemoteHelperError> {
        trace!(
            "getting address: {} in {}",
            remote_name,
            self.path.to_string_lossy()
        );
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["remote", "get-url", remote_name])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting remote url".to_string(),
                details: Some(e.to_string()),
            })?;
        if !output.status.success() {
            return Err(RemoteHelperError::Failure {
                action: "getting remote url".to_string(),
                details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        let remote_url =
            String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
                action: "reading stdout of git remote get-url".to_string(),
                details: Some(e.to_string()),
            })?;
        let remote_url = remote_url.trim();

        let prefix = format!("{}://0x", protocol);
        let address_str = remote_url
            .strip_prefix(&prefix)
            .ok_or(RemoteHelperError::Failure {
                action: "getting address".to_string(),
                details: Some(format!("address not found in {}", remote_url)),
            })?;
        let address = hex::decode(address_str).map_err(|e| RemoteHelperError::Failure {
            action: "decoding address".to_string(),
            details: Some(e.to_string()),
        })?;
        let address: &[u8; 20] = address.as_array().ok_or(RemoteHelperError::Failure {
            action: "getting address".to_string(),
            details: None,
        })?;
        debug!("got address: {}", address_str);
        Ok(*address)
    }

    fn resolve_reference(&self, name: &str) -> Result<Hash, RemoteHelperError> {
        trace!(
            "resolving reference: {} in {}",
            name,
            self.path.to_string_lossy()
        );
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["rev-parse", name])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "resolving reference".to_string(),
                details: Some(e.to_string()),
            })?;
        if !output.status.success() {
            return Err(RemoteHelperError::Failure {
                action: "resolving reference".to_string(),
                details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }
        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git rev-parse".to_string(),
            details: Some(e.to_string()),
        })?;
        let hash = Hash::from_str(stdout.trim()).map_err(|e| RemoteHelperError::Failure {
            action: "parsing hash".to_string(),
            details: Some(e.to_string()),
        })?;
        debug!("resolved reference {}: {}", name, hash);
        Ok(hash)
    }

    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        trace!(
            "getting object: {} in {}",
            hash,
            self.path.to_string_lossy()
        );
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["cat-file", "-t", &hash.to_string()])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting object type".to_string(),
                details: Some(e.to_string()),
            })?;
        if !output.status.success() {
            return Err(RemoteHelperError::Failure {
                action: "getting object type".to_string(),
                details: Some(format!("git cat-file -t {} failed", hash)),
            });
        }
        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git cat-file".to_string(),
            details: Some(e.to_string()),
        })?;
        let kind = ObjectKind::from_str(stdout.trim()).map_err(|e| RemoteHelperError::Failure {
            action: "parsing object type".to_string(),
            details: Some(e.to_string()),
        })?;

        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&["cat-file", kind.to_string().as_str(), &hash.to_string()])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting object type".to_string(),
                details: Some(e.to_string()),
            })?;
        let object = Object::new(kind, output.stdout, hash.is_sha256())?;
        debug!("got object {}: {}", hash, object.get_kind());

        if hash != object.hash(self.is_sha256()?) {
            return Err(RemoteHelperError::Failure {
                action: "getting object".to_string(),
                details: Some(format!("object hash mismatch: {} != {}", hash, object.hash(self.is_sha256()?)),
                ),
            });
        }

        Ok(object)
    }

    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError> {
        trace!(
            "saving object: {} in {}",
            object.get_kind(),
            self.path.to_string_lossy()
        );
        let mut cmd = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&[
                "hash-object",
                "-t",
                &object.get_kind().to_string(),
                "-w",
                "--stdin",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| RemoteHelperError::Failure {
                action: "saving object".to_string(),
                details: Some(e.to_string()),
            })?;

        cmd.stdin
            .take()
            .ok_or(RemoteHelperError::Failure {
                action: "saving object".to_string(),
                details: Some("failed to get stdin".to_string()),
            })?
            .write_all(object.get_data())
            .map_err(|e| RemoteHelperError::Failure {
                action: "writing object to stdin".to_string(),
                details: Some(e.to_string()),
            })?;

        let output = cmd
            .wait_with_output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting object hash".to_string(),
                details: Some(e.to_string()),
            })?;

        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).map_err(|e| RemoteHelperError::Failure {
                    action: "reading stderr of git hash-object".to_string(),
                    details: Some(e.to_string()),
                })?;
            return Err(RemoteHelperError::Failure {
                action: "saving object".to_string(),
                details: Some(stderr),
            });
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| RemoteHelperError::Failure {
            action: "reading stdout of git hash-object".to_string(),
            details: Some(e.to_string()),
        })?;
        let hash = Hash::from_str(stdout.trim()).map_err(|e| RemoteHelperError::Failure {
            action: "parsing saved object's hash".to_string(),
            details: Some(e.to_string()),
        })?;

        let object_hash = object.hash(self.is_sha256()?);
        if hash != object_hash {
            return Err(RemoteHelperError::Failure {
                action: "saving object".to_string(),
                details: Some(format!("object hash mismatch: {} != {}", hash, object_hash)),
            });
        }
        debug!("saved object: {}", hash);

        Ok(())
    }

    fn list_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        let range = format!("{}..{}", remote, local);
        trace!(
            "listing missing objects: {} in {}",
            range,
            self.path.to_string_lossy()
        );
        let hashes = self.rev_list(&range)?;
        debug!("got missing objects: {:?}", hashes);
        Ok(hashes)
    }

    fn list_objects(&self, hash: Hash) -> Result<Vec<Hash>, RemoteHelperError> {
        trace!(
            "listing objects: {} in {}",
            hash,
            self.path.to_string_lossy()
        );
        let hashes = self.rev_list(&hash.to_string())?;
        debug!("got objects: {:?}", hashes);
        Ok(hashes)
    }
}

#[cfg(test)]
fn setup_git_repo(is_sha256: bool) -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

    let object_format = if is_sha256 { "sha256" } else { "sha1" };

    let output = Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init", &format!("--object-format={}", object_format)])
        .output()
        .expect("failed to run git init");
    if !output.status.success() {
        panic!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    temp_dir
}

#[test]
fn test_resolve_reference() {
    let repo_dir = setup_git_repo(true);

    let mut file =
        std::fs::File::create(repo_dir.path().join("abc")).expect("failed to create abc file");
    file.write_all(b"example").expect("failed to write abc");

    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["add", "abc"])
        .output()
        .expect("failed to run git add");
    if !cmd.status.success() {
        panic!("git add failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["commit", "-m", "something"])
        .output()
        .expect("failed to run git hash-object");
    if !cmd.status.success() {
        panic!(
            "git commit failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let hash = git
        .resolve_reference("HEAD")
        .expect("failed to resolve reference");
    assert_eq!(hash, get_head_hash(&repo_dir));
}

#[test]
fn test_save_object() {
    let repo_dir = setup_git_repo(true);
    let git = SystemGit::new(repo_dir.path().to_path_buf());

    let data = b"test";
    let object = Object::new(ObjectKind::Blob, data.to_vec(), true).expect("failed to create object");
    git.save_object(object).expect("failed to save object");
}

#[cfg(test)]
fn get_head_hash(repo_dir: &tempfile::TempDir) -> Hash {
    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("failed to run git rev-parse");
    if !cmd.status.success() {
        let stderr = String::from_utf8_lossy(&cmd.stderr);
        panic!("git rev-parse failed: {}", stderr);
    }
    let stdout = String::from_utf8(cmd.stdout).expect("failed to convert stdout to string");
    Hash::from_str(stdout.trim()).expect("failed to parse hash")
}

#[cfg(test)]
fn commit_file(repo_dir: &tempfile::TempDir, file_name: &str, content: &[u8]) {
    let mut file =
        std::fs::File::create(repo_dir.path().join(file_name)).expect("failed to create abc file");
    file.write_all(content).expect("failed to write abc");

    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["add", file_name])
        .output()
        .expect("failed to run git add");
    if !cmd.status.success() {
        panic!("git add failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["commit", "-m", "something"])
        .output()
        .expect("failed to run git hash-object");
    if !cmd.status.success() {
        panic!(
            "git commit failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&cmd.stdout),
            String::from_utf8_lossy(&cmd.stderr)
        );
    }
}

#[test]
fn test_get_object() {
    let repo_dir = setup_git_repo(false);

    let blob0_content = b"example";
    let blob1_content = b"example2";
    commit_file(&repo_dir, "abc", blob0_content);
    commit_file(&repo_dir, "def", blob1_content);

    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let object = git
        .get_object(get_head_hash(&repo_dir))
        .expect("failed to get object");
    assert_eq!(object.get_kind(), &ObjectKind::Commit);
    let related_objects = object.get_related_objects();
    assert_eq!(related_objects.len(), 2);

    let object = git
        .get_object(related_objects[0].clone())
        .expect("failed to get tree object");
    assert_eq!(object.get_kind(), &ObjectKind::Tree);
    let related_objects = object.get_related_objects();
    assert_eq!(related_objects.len(), 2);

    let blob0 = git
        .get_object(related_objects[0].clone())
        .expect("failed to get blob object");
    assert_eq!(blob0.get_kind(), &ObjectKind::Blob);
    assert_eq!(blob0.get_data(), blob0_content);

    let blob1 = git
        .get_object(related_objects[1].clone())
        .expect("failed to get blob object");
    assert_eq!(blob1.get_kind(), &ObjectKind::Blob);
    assert_eq!(blob1.get_data(), blob1_content);
}

#[test]
fn test_list_missing_objects() {
    let repo_dir = setup_git_repo(true);

    let mut file =
        std::fs::File::create(repo_dir.path().join("abc")).expect("failed to create abc file");
    file.write_all(b"example").expect("failed to write abc");

    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["add", "abc"])
        .output()
        .expect("failed to run git add");
    if !cmd.status.success() {
        panic!("git add failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["commit", "-m", "first commit"])
        .output()
        .expect("failed to run git hash-object");
    if !cmd.status.success() {
        panic!(
            "git commit failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }
    let hash_before = get_head_hash(&repo_dir);

    file.write_all(b"example2").expect("failed to write abc");
    let cmd = Command::new("git")
        .current_dir(repo_dir.path())
        .args(&["commit", "-am", "second commit"])
        .output()
        .expect("failed to run git add");
    if !cmd.status.success() {
        panic!("git add failed: {}", String::from_utf8_lossy(&cmd.stderr));
    }
    let hash_after = get_head_hash(&repo_dir);

    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let missing = git
        .list_missing_objects(hash_after.clone(), hash_before)
        .expect("failed to get missing objects");

    assert_eq!(missing.len(), 3); // blob, tree, commit
    assert!(missing.contains(&hash_after));
}

#[test]
fn test_get_address() {
    let repo_dir = setup_git_repo(true);
    let git = SystemGit::new(repo_dir.path().to_path_buf());

    let add_remote = |remote_name: &str, url: &str| {
        let cmd = Command::new("git")
            .current_dir(repo_dir.path())
            .args(&["remote", "add", remote_name, url])
            .output()
            .expect("failed to run git remote add");
        if !cmd.status.success() {
            panic!(
                "git remote add failed: {}",
                String::from_utf8_lossy(&cmd.stderr)
            );
        }
    };

    add_remote("origin", "eth://0x0000000000000000000000000000000000000000");
    let address = git
        .get_address("eth", "origin")
        .expect("failed to get address");
    assert_eq!(
        hex::encode(address),
        "0000000000000000000000000000000000000000"
    );

    add_remote(
        "upstream",
        "arb1://0xc6093fd9cc143f9f058938868b2df2daf9a91d28",
    );
    let address = git
        .get_address("arb1", "upstream")
        .expect("failed to get address");
    assert_eq!(
        hex::encode(address).to_lowercase(),
        "c6093fd9cc143f9f058938868b2df2daf9a91d28"
    );
}

#[test]
fn test_get_version() {
    let repo_dir = setup_git_repo(true);
    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let version = git.version().expect("failed to get version");
    assert!(version.major >= 1);
}

#[test]
fn test_is_sha256() {
    let repo_dir = setup_git_repo(true);
    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let is_sha256 = git.is_sha256().expect("failed to get is_sha256");
    assert!(is_sha256);

    let repo_dir = setup_git_repo(false);
    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let is_sha256 = git.is_sha256().expect("failed to get is_sha256");
    assert!(!is_sha256);
}
