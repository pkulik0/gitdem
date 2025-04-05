use std::error::Error;
use std::path::Path;
use std::process::Command;

static PACKAGE_MANAGER: &str = "npm";

fn check_package_manager() -> Result<(), Box<dyn Error>> {
    let output = Command::new(PACKAGE_MANAGER).arg("--version").output()?;
    if !output.status.success() {
        return Err("package manager command returned non-zero exit code".into());
    }
    Ok(())
}

fn build_wallet_bridge(project_root: &Path) -> Result<(), Box<dyn Error>> {
    let bridge_path = project_root.join("wallet-bridge");
    println!("cargo:rerun-if-changed={}", bridge_path.display());

    println!(
        "installing wallet bridge dependencies at {}",
        bridge_path.display()
    );
    let output = Command::new(PACKAGE_MANAGER)
        .current_dir(bridge_path.as_path())
        .arg("install")
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    println!("building wallet bridge at {}", bridge_path.display());
    let output = Command::new(PACKAGE_MANAGER)
        .current_dir(bridge_path)
        .args(&["run", "build"])
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    Ok(())
}

fn build_on_chain(project_root: &Path) -> Result<(), Box<dyn Error>> {
    let on_chain_path = project_root.join("on-chain");
    println!("cargo:rerun-if-changed={}", on_chain_path.display());

    println!(
        "installing on-chain dependencies at {}",
        on_chain_path.display()
    );
    let output = Command::new(PACKAGE_MANAGER)
        .current_dir(on_chain_path.as_path())
        .arg("install")
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    println!(
        "building on-chain contract(s) at {}",
        on_chain_path.display()
    );
    let output = Command::new("npx")
        .current_dir(on_chain_path.as_path())
        .args(&["hardhat", "compile"])
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    Ok(())
}

fn exit_with_error(message: &str, error: Box<dyn Error>) -> ! {
    eprintln!("{}: {}", message, error);
    std::process::exit(1);
}

fn main() {
    match check_package_manager() {
        Ok(_) => println!("package manager found"),
        Err(e) => exit_with_error("package manager not found", e),
    }

    let manifest_dir_var: String =
        std::env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest dir");
    let project_root = Path::new(&manifest_dir_var)
        .parent()
        .expect("failed to get project root");

    match build_wallet_bridge(project_root) {
        Ok(_) => println!("wallet bridge built"),
        Err(e) => exit_with_error("failed to build wallet bridge", e),
    }

    match build_on_chain(project_root) {
        Ok(_) => println!("on-chain contract(s) built"),
        Err(e) => exit_with_error("failed to build on-chain contract(s)", e),
    }
}
