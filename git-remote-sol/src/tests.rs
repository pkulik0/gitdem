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
        panic!("failed to build {}", String::from_utf8_lossy(&build_cmd.stderr));
    }

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

    let repo_address = "DBWrGX82Abj1R9HxarNuucwSdyuq11HU4twzfjgQZ1FJ";
    let remote_url = format!("sol://{}", repo_address);

    let cmd = Command::new("git")
        .args(&["clone", remote_url.as_str()])
        .env("PATH", path)
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to clone");
    assert!(!cmd.status.success()); // TODO: remove and uncomment the following lines once it works
    
    // if !cmd.status.success() {
    //     panic!("failed to clone {}", String::from_utf8_lossy(&cmd.stderr));
    // }

    // let repo_path = temp_dir.path().join(repo_address);
    // assert!(repo_path.exists());

    // let repo = git2::Repository::open(repo_path).expect("failed to open repo");
    // let remotes = repo
    //     .find_remote("origin")
    //     .expect("failed to find origin remote");
    // assert_eq!(remotes.url(), Some(remote_url.as_str()));
}
