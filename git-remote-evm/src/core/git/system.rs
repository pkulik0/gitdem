use crate::core::git::Git;
use crate::core::hash::Hash;
use crate::core::object::Object;
use crate::core::remote_helper::error::RemoteHelperError;
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

    #[cfg(test)]
    pub fn init(&self) -> Result<(), RemoteHelperError> {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .args(&["init", "--object-format=sha256"])
            .output()
            .map_err(|e| RemoteHelperError::Failure {
                action: "initializing git repo".to_string(),
                details: Some(e.to_string()),
            })?;
        if !output.status.success() {
            let stderr =
                String::from_utf8(output.stderr).map_err(|e| RemoteHelperError::Failure {
                    action: "reading stderr of git init".to_string(),
                    details: Some(e.to_string()),
                })?;
            return Err(RemoteHelperError::Failure {
                action: "initializing git repo".to_string(),
                details: Some(stderr),
            });
        }
        Ok(())
    }
}

impl Git for SystemGit {
    fn get_object(&self, hash: Hash) -> Result<Object, RemoteHelperError> {
        todo!()
    }

    fn save_object(&self, object: Object) -> Result<(), RemoteHelperError> {
        let mut cmd = Command::new("git")
            .current_dir(self.path.as_path())
            .args(&["hash-object", "-w", "--stdin"])
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

        // TODO: uncomment after objects hash properly
        // if hash != object.hash {
        //     return Err(RemoteHelperError::Failure {
        //         action: "saving object".to_string(),
        //         details: Some(format!("object hash mismatch: {} != {}", hash, object.hash)),
        //     });
        // }

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
    let git = SystemGit::new(temp_dir.path().to_path_buf());
    git.init().expect("failed to initialize git repo");
    temp_dir
}

#[test]
fn test_save_object() {
    let repo_dir = setup_git_repo();
    let git = SystemGit::new(repo_dir.path().to_path_buf());

    let data = b"test";
    let hash = Hash::from_data_sha256(data).expect("failed to create hash");
    let object = Object::new(hash, data.to_vec());
    git.save_object(object).expect("failed to save object");
}
