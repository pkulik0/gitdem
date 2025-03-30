mod cli;
mod config;
mod remote_helper;

use cli::CLI;
use config::git::GitConfig;

#[cfg(test)]
mod tests;

#[cfg(feature = "mock")]
use config::mock::MockConfig;
#[cfg(feature = "mock")]
use remote_helper::mock::Mock;
// #[cfg(feature = "mock")]
// use remote_helper::reference::{Reference, Value};

use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{error, info};
use remote_helper::solana::helper::Solana;
use std::io;
use std::io::Write;

fn main() {
    let _logger = Logger::try_with_str("trace").expect("failed to create logger")
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .expect("failed to start logger");

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let remote_name = std::env::args().nth(1).unwrap_or_default();
    let remote_url = std::env::args().nth(2).unwrap_or_default();

    #[cfg(not(feature = "mock"))]
    let config = Box::new(GitConfig::new());
    #[cfg(feature = "mock")]
    let config = Box::new(MockConfig::new());

    #[cfg(not(feature = "mock"))]
    let remote_helper = Box::new(Solana::new(config));
    #[cfg(feature = "mock")]
    let remote_helper = Box::new(Mock::new());

    let mut cli = CLI::new(
        remote_helper,
        &mut stdin,
        &mut stdout,
        &mut stderr,
        remote_name,
        remote_url,
    );

    match cli.run() {
        Ok(_) => {},
        Err(e) => {
            error!("failed to run cli: {}", e);
            writeln!(stderr, "remote-sol: {}", e).unwrap();
            std::process::exit(1);
        }
    }
}
