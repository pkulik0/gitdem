use super::remote_helper::error::RemoteHelperError;
use crate::core::hash::Hash;
use crate::core::object::{Object, ObjectKind};
use log::{debug, trace};
use mockall::automock;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[automock]
pub trait Git {
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
    fn rev_parse(&self, name: &str) -> Result<Vec<Hash>, RemoteHelperError> {
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
            .args(&["cat-file", "-p", &hash.to_string()])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting object type".to_string(),
                details: Some(e.to_string()),
            })?;
        let object = Object {
            kind,
            data: output.stdout,
        };
        debug!("got object {}: {}", hash, object.kind);
        Ok(object)
    }

    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError> {
        trace!(
            "saving object: {} in {}",
            object.kind,
            self.path.to_string_lossy()
        );
        let mut cmd = Command::new("git")
            .current_dir(self.path.as_path())
            .env_remove("GIT_DIR")
            .args(&[
                "hash-object",
                "-t",
                &object.kind.to_string(),
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
            .write_all(&object.data)
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

        let object_hash = object.hash(true);
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
        let range = format!("{}..{}", local, remote);
        trace!(
            "listing missing objects: {} in {}",
            range,
            self.path.to_string_lossy()
        );
        let hashes = self.rev_parse(&range)?;
        debug!("got missing objects: {:?}", hashes);
        Ok(hashes)
    }

    fn list_objects(&self, hash: Hash) -> Result<Vec<Hash>, RemoteHelperError> {
        trace!(
            "listing objects: {} in {}",
            hash,
            self.path.to_string_lossy()
        );
        let hashes = self.rev_parse(&hash.to_string())?;
        debug!("got objects: {:?}", hashes);
        Ok(hashes)
    }
}

#[cfg(test)]
fn setup_git_repo() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new("git")
        .current_dir(temp_dir.path())
        .args(&["init", "--object-format=sha256"])
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
    let repo_dir = setup_git_repo();

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
    let repo_dir = setup_git_repo();
    let git = SystemGit::new(repo_dir.path().to_path_buf());

    let data = b"test";
    let object = Object {
        kind: ObjectKind::Blob,
        data: data.to_vec(),
    };
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

#[test]
fn test_get_object() {
    let repo_dir = setup_git_repo();

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
    let object = git
        .get_object(get_head_hash(&repo_dir))
        .expect("failed to get object");
    assert_eq!(object.kind, ObjectKind::Commit);
    let commit_data =
        String::from_utf8(object.data).expect("failed to convert object data to string");
    let tree_data = commit_data
        .split('\n')
        .next()
        .expect("failed to get tree data");
    let tree_hash_str = tree_data
        .strip_prefix("tree ")
        .expect("failed to strip tree prefix");
    let tree_hash = Hash::from_str(tree_hash_str).expect("failed to parse tree hash");

    let object = git
        .get_object(tree_hash)
        .expect("failed to get tree object");
    assert_eq!(object.kind, ObjectKind::Tree);
    let tree_data =
        String::from_utf8(object.data).expect("failed to convert object data to string");
    let tree_entries = tree_data
        .split('\n')
        .next()
        .expect("failed to get tree entries");
    let blob_hash_str = tree_entries
        .strip_prefix("100644 blob ")
        .expect("failed to strip blob prefix");
    let blob_hash_str = blob_hash_str
        .strip_suffix("\tabc")
        .expect("failed to strip blob suffix");
    let blob_hash = Hash::from_str(blob_hash_str).expect("failed to parse blob hash");

    let object = git
        .get_object(blob_hash)
        .expect("failed to get blob object");
    assert_eq!(object.kind, ObjectKind::Blob);
    assert_eq!(object.data, b"example");
}

#[test]
fn test_list_missing_objects() {
    let repo_dir = setup_git_repo();

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
        .list_missing_objects(hash_before, hash_after.clone())
        .expect("failed to get missing objects");

    assert_eq!(missing.len(), 3); // blob, tree, commit
    assert!(missing.contains(&hash_after));
}

#[test]
fn test_get_address() {
    let repo_dir = setup_git_repo();
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
