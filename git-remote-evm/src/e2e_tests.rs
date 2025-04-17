use std::{io::Write, path::PathBuf, process::Command};

fn deploy_contract(manifest_dir: &PathBuf) -> String {
    let on_chain_dir = manifest_dir
        .parent()
        .expect("failed to get parent")
        .join("on-chain");

    let output = Command::new("npx")
        .args(&[
            "hardhat",
            "ignition",
            "deploy",
            "ignition/modules/GitRepository.ts",
            "--network",
            "localhost",
        ])
        .current_dir(on_chain_dir)
        .output()
        .expect("failed to deploy contract");
    if !output.status.success() {
        panic!(
            "failed to deploy contract {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).expect("failed to convert stdout to string");
    let address = stdout
        .lines()
        .find(|line| line.contains("GitRepositoryModule#GitRepository -"))
        .expect("failed to find address")
        .split_whitespace()
        .nth(2)
        .expect("failed to find address");
    address.to_string()
}

fn build_and_link(manifest_dir: &PathBuf) -> String {
    // 1. Ensure the binary is compiled
    let build_cmd = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(manifest_dir)
        .output()
        .expect("failed to build");
    if !build_cmd.status.success() {
        panic!(
            "failed to build {}",
            String::from_utf8_lossy(&build_cmd.stderr)
        );
    }

    // 2. Symlink git-remote-evm to git-remote-eth
    let target_dir = manifest_dir.join("target/release");
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

fn prepare() -> (tempfile::TempDir, String, impl Fn() -> Command) {
    let manifest_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").expect("has to be set"));

    let path = build_and_link(&manifest_dir);
    let repo_address = deploy_contract(&manifest_dir);

    let repo_dir = tempfile::tempdir().expect("failed to create temp dir");
    let repo_path = repo_dir.path().to_path_buf(); // for closure
    let command_builder = move || {
        let mut cmd = Command::new("git");
        cmd.env("PATH", path.as_str())
            .env("GITDEM_WALLET", "environment")
            .env(
                "GITDEM_PRIVATE_KEY",
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            )
            .env("GITDEM_ETH_RPC", "http://127.0.0.1:8545/")
            .current_dir(&repo_path);
        cmd
    };

    let cmd = command_builder()
        .args(&["init"])
        .output()
        .expect("failed to init");
    if !cmd.status.success() {
        panic!("failed to init {}", String::from_utf8_lossy(&cmd.stderr));
    }
    // GitHub Actions CI seems to have an outdated version that still uses 'master' as the default branch name
    let cmd = command_builder()
        .args(&["checkout", "-b", "main"])
        .output()
        .expect("failed to checkout main");
    if !cmd.status.success() {
        panic!(
            "failed to checkout main {}",
            String::from_utf8_lossy(&cmd.stderr)
        );
    }

    (repo_dir, repo_address, command_builder)
}

#[test]
fn clone_empty() {
    let (repo_dir, repo_address, build_cmd) = prepare();

    let output = build_cmd()
        .args(&["clone", format!("eth://{}", repo_address).as_str()])
        .output()
        .expect("failed to clone");
    if !output.status.success() {
        panic!(
            "failed to clone: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let repo_path = repo_dir.path().join(repo_address);
    assert!(repo_path.exists());
}

#[test]
fn push_simple() {
    let (repo_dir, repo_address, build_cmd) = prepare();

    let file_name = "test.txt";
    let mut file =
        std::fs::File::create(repo_dir.path().join(file_name)).expect("failed to create file");
    file.write_all(b"test").expect("failed to write to file");

    let output = build_cmd()
        .args(&["add", file_name])
        .output()
        .expect("failed to add file");
    if !output.status.success() {
        panic!(
            "failed to add file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = build_cmd()
        .args(&["commit", "-m", "test"])
        .output()
        .expect("failed to commit");
    if !output.status.success() {
        panic!(
            "failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = build_cmd()
        .args(&[
            "remote",
            "add",
            "origin",
            format!("eth://{}", repo_address).as_str(),
        ])
        .output()
        .expect("failed to add remote");
    if !output.status.success() {
        panic!(
            "failed to add remote: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = build_cmd()
        .args(&["push", "--set-upstream", "origin", "main"])
        .output()
        .expect("failed to push");
    if !output.status.success() {
        panic!(
            "failed to push: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
