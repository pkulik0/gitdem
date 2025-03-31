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
#[cfg(feature = "mock")]
use remote_helper::reference::{Keyword, Reference, Value};

use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{debug, error, info, warn};
use remote_helper::solana::helper::Solana;
use std::io;
use std::io::Write;

// Remote helpers are run by git
// Use this environment variable to wait for a debugger to attach
#[cfg(debug_assertions)]
static DEBUG_ENV_VAR: &str = "DEBUG_WAIT";

fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"couldn't get panic payload");
        let location = panic_info.location().map_or_else(
            || "unknown location".to_string(),
            |loc| format!("{}:{}", loc.file(), loc.line()),
        );
        error!("panic at {}, payload: {:?}", location, payload);
        default_hook(panic_info);
    }));
}

fn main() {
    let _logger = Logger::try_with_str("trace")
        .expect("failed to create logger")
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::Direct)
        .start()
        .expect("failed to start logger");

    setup_panic_hook();

    #[cfg(debug_assertions)]
    if std::env::var(DEBUG_ENV_VAR).is_ok() {
        debug!("waiting for debugger to attach");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let remote_name = std::env::args().nth(1).unwrap_or_default();
    let remote_url = std::env::args().nth(2).unwrap_or_default();

    #[cfg(feature = "mock")]
    warn!("using mock remote helper");

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
        Ok(_) => {}
        Err(e) => {
            error!("failed to run cli: {}", e);
            writeln!(stderr, "remote-sol: {}", e).expect("failed to write to stderr");
            std::process::exit(1);
        }
    }
}
