use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::object::{Object, ObjectKind};
use crate::core::remote_helper::error::RemoteHelperError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

pub struct SystemGit {
    path: PathBuf,
}

impl SystemGit {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Git for SystemGit {
    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
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
            .args(&["cat-file", "-p", &hash.to_string()])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "getting object type".to_string(),
                details: Some(e.to_string()),
            })?;
        Ok(Object {
            kind,
            data: output.stdout,
        })
    }

    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError> {
        let mut cmd = Command::new("git")
            .current_dir(self.path.as_path())
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

        Ok(())
    }

    fn get_missing_objects(
        &self,
        local: Hash,
        remote: Hash,
    ) -> Result<Vec<Hash>, RemoteHelperError> {
        todo!()
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

#[test]
fn test_get_object() {
    let repo_dir = setup_git_repo();

    let mut file = File::create(repo_dir.path().join("abc")).expect("failed to create abc file");
    file.write_all(b"example").expect("failed to write abc");
    drop(file);

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
    let hash = Hash::from_str(stdout.trim()).expect("failed to parse hash");

    let git = SystemGit::new(repo_dir.path().to_path_buf());
    let object = git.get_object(hash).expect("failed to get object");
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
