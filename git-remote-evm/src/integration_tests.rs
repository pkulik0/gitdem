use std::process::Command;

fn get_path_and_prepare() -> String {
    // 1. Ensure the binary is compiled
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let build_cmd = Command::new("cargo")
        .args(&["build", "--release", "--features", "mock"])
        .current_dir(&manifest_dir)
        .output()
        .expect("failed to build");
    if !build_cmd.status.success() {
        panic!(
            "failed to build {}",
            String::from_utf8_lossy(&build_cmd.stderr)
        );
    }

    // 2. Symlink git-remote-evm to git-remote-eth
    let target_dir = std::path::Path::new(&manifest_dir).join("target/release");
    let evm_path = target_dir.join("git-remote-evm");
    let eth_path = target_dir.join("git-remote-eth");
    if let Err(e) = std::os::unix::fs::symlink(evm_path, eth_path) {
        if !e.to_string().contains("exists") {
            panic!("failed to link git-remote-evm to git-remote-eth: {}", e);
        }
    }

    // 3. Prepare a new PATH with the target/release/ as the first match
    let path = std::env::var("PATH").expect("PATH is not set");
    let new_path = format!("{}:{}", target_dir.display(), path);
    new_path
}

fn init_git_repo(path_envvar: &str, temp_dir: &tempfile::TempDir) {
    let cmd = Command::new("git")
        .args(&["init"])
        .env("PATH", path_envvar)
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to init");
    if !cmd.status.success() {
        panic!("failed to init {}", String::from_utf8_lossy(&cmd.stderr));
    }

    // GitHub Actions CI seems to have an outdated version that still uses 'master' as the default branch name
    let cmd = Command::new("git")
        .args(&["checkout", "-b", "main"])
        .env("PATH", path_envvar)
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to checkout main");
    let stderr = String::from_utf8_lossy(&cmd.stderr);
    if !cmd.status.success() && !stderr.contains("already exists") {
        panic!("failed to checkout main {}", stderr);
    }
}

// #[test]
// fn test_integration_clone() {
//     let path = get_path_and_prepare();
//     let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

//     let repo_address = "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ";
//     let remote_url = format!("sol://{}", repo_address);

//     let cmd = Command::new("git")
//         .args(&["clone", remote_url.as_str()])
//         .env("PATH", path)
//         .current_dir(temp_dir.path())
//         .output()
//         .expect("failed to clone");
//     if !cmd.status.success() {
//         panic!("failed to clone {}", String::from_utf8_lossy(&cmd.stderr));
//     }

//     let repo_path = temp_dir.path().join(repo_address);
//     assert!(repo_path.exists());

//     let repo = git2::Repository::open(repo_path).expect("failed to open repo");
//     let remotes = repo
//         .find_remote("origin")
//         .expect("failed to find origin remote");
//     assert_eq!(remotes.url(), Some(remote_url.as_str()));
// }

#[test]
fn test_integration_push() {
    let path = get_path_and_prepare();
    let repo_dir = tempfile::tempdir().expect("failed to create temp dir");

    let repo_address = "0xc0ffee254729296a45a3885639AC7E10F9d54979";
    let remote_url = format!("eth://{}", repo_address);

    init_git_repo(&path, &repo_dir);

    let file_content = "Hello, world!";
    let file_path = repo_dir.path().join("test.txt");
    std::fs::write(file_path, file_content).expect("failed to write file");

    let cmd = Command::new("git")
        .args(&["add", "test.txt"])
        .env("PATH", path.as_str())
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to add");
    if !cmd.status.success() {
        panic!("failed to add {}", String::from_utf8_lossy(&cmd.stderr));
    }

    let cmd = Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .env("PATH", path.as_str())
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to commit");
    if !cmd.status.success() {
        panic!("failed to commit {}", String::from_utf8_lossy(&cmd.stderr));
    }

    let remote_name = "origin";
    let cmd = Command::new("git")
        .args(&["remote", "add", remote_name, remote_url.as_str()])
        .env("PATH", path.as_str())
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to add remote");
    if !cmd.status.success() {
        panic!(
            "failed to add remote {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    let cmd = Command::new("git")
        .args(&["push", "--set-upstream", remote_name, "main"])
        .env("PATH", path.as_str())
        .current_dir(repo_dir.path())
        .output()
        .expect("failed to push");
    if !cmd.status.success() {
        panic!("failed to push {}", String::from_utf8_lossy(&cmd.stderr));
    }
}
