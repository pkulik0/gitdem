use std::process::Command;

fn get_path_and_prepare() -> String {
  // 1. Ensure the binary is compiled
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
  let build_cmd = Command::new("cargo")
    .args(&["build", "--release"])
    .current_dir(&manifest_dir)
    .output()
    .expect("failed to build");
  assert!(build_cmd.status.success());

  // 2. Prepare a new PATH with the target/release/git-remote-sol as the first match
  let path = std::env::var("PATH").expect("PATH is not set");
  let target_dir = std::path::Path::new(&manifest_dir).join("target/release");
  let new_path = format!("{}:{}", target_dir.display(), path);
  new_path
}

#[test]
fn test_integration_clone() {
    let path = get_path_and_prepare();

    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let temp_dir_path = temp_dir.path().display().to_string();

    let repo_name = "test-repo";
    let cmd = Command::new("git")
      .args(&["clone", &format!("sol://{}", repo_name)])
      .env("PATH", path)
      .current_dir(temp_dir_path.clone())
      .output() 
      .expect("failed to clone");
    assert!(cmd.status.success());

    let repo_path = std::path::Path::new(&temp_dir_path).join(repo_name);
    assert!(repo_path.exists());
}