use std::error::Error;
use std::process::Command;

static PACKAGE_MANAGER: &str = "yarn";

fn check_package_manager() -> Result<(), Box<dyn Error>> {
    let output = Command::new(PACKAGE_MANAGER).arg("--version").output()?;
    if !output.status.success() {
        return Err("package manager command returned non-zero exit code".into());
    }
    Ok(())
}

fn build_wallet_bridge() -> Result<(), Box<dyn Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let bridge_path = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("couldn't find parent directory")
        .join("wallet-bridge");
    println!("cargo:rerun-if-changed={}", bridge_path.display());

    println!(
        "installing wallet bridge dependencies at {}",
        bridge_path.display()
    );
    let output = Command::new(PACKAGE_MANAGER)
        .current_dir(bridge_path.clone())
        .arg("install")
        .output()?;
    if !output.status.success() {
        return Err("failed to install wallet bridge dependencies".into());
    }

    println!("building wallet bridge at {}", bridge_path.display());
    let output = Command::new(PACKAGE_MANAGER)
        .current_dir(bridge_path)
        .arg("build")
        .output()?;
    if !output.status.success() {
        return Err("failed to build wallet bridge".into());
    }

    Ok(())
}

fn main() {
    match check_package_manager() {
        Ok(_) => println!("package manager found"),
        Err(e) => {
            eprintln!("package manager not found: {}", e);
            std::process::exit(1);
        }
    }
    match build_wallet_bridge() {
        Ok(_) => println!("wallet bridge built"),
        Err(e) => {
            eprintln!("failed to build wallet bridge: {}", e);
            std::process::exit(1);
        }
    }
}
