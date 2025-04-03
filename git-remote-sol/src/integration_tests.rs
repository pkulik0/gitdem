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

    // 2. Prepare a new PATH with the target/release/git-remote-sol as the first match
    let path = std::env::var("PATH").expect("PATH is not set");
    let target_dir = std::path::Path::new(&manifest_dir).join("target/release");
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

    let repo_address = "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ";
    let remote_url = format!("sol://{}", repo_address);

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
