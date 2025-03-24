use std::path::Path;
use std::process::Command;
use std::error::Error;

static PACKAGE_MANAGER: &str = "yarn";

fn build_wallet_bridge() -> Result<(), Box<dyn Error>> {
    Command::new(PACKAGE_MANAGER)
      .current_dir(Path::new("../wallet-bridge"))
      .arg("build")
      .output()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    build_wallet_bridge()?;
    Ok(())
}
