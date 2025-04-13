#![feature(slice_as_array)]

mod args;
mod cli;
mod core;
#[cfg(test)]
mod e2e_tests;

use args::Args;
use cli::CLI;
use core::git::Git;
use core::remote_helper::{error::RemoteHelperError, evm::Evm};
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{debug, error};
use std::error::Error;
use std::io;
use std::path::PathBuf;

// Remote helpers are run by git
// Use this environment variable to wait for a debugger to attach
#[cfg(debug_assertions)]
const DEBUG_ENV_VAR: &str = "DEBUG_WAIT";
const GIT_DIR_ENV_VAR: &str = "GIT_DIR";

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

fn construct_remote_helper(args: Args) -> Result<Evm, RemoteHelperError> {
    use core::git::SystemGit;
    use core::kv_source::GitConfigSource;
    use core::remote_helper::{config::Config, executor::create_executor};

    debug!("using evm remote helper");
    let kv_source = Box::new(GitConfigSource::new(args.directory().clone()));
    let git = Box::new(SystemGit::new(args.directory().clone()));

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| RemoteHelperError::Failure {
            action: "creating runtime".to_string(),
            details: Some(e.to_string()),
        })?;
    let config = Config::new(args.protocol().to_string(), kv_source);

    let address = if let Some(address) = args.address() {
        *address
    } else {
        git.get_address(
            args.protocol(),
            args.remote_name().ok_or(RemoteHelperError::Missing {
                what: "remote name".to_string(),
            })?,
        )?
    };

    let executor = runtime.block_on(create_executor(
        &config.get_rpc()?,
        config.get_wallet()?,
        address,
    ))?;

    Evm::new(runtime, executor, git)
}

fn exit_with_error(msg: &str, e: Box<dyn Error>) -> ! {
    error!("{}: {}", msg, e);
    eprintln!("remote: {}", e);
    std::process::exit(1);
}

fn main() {
    let _logger = Logger::try_with_str("trace")
        .expect("failed to create logger")
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::Direct)
        .start()
        .unwrap_or_else(|e| exit_with_error("failed to start logger", e.into()));

    setup_panic_hook();

    #[cfg(debug_assertions)]
    if std::env::var(DEBUG_ENV_VAR).is_ok() {
        debug!("waiting for debugger to attach");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let git_dir = match std::env::var(GIT_DIR_ENV_VAR) {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => PathBuf::from("."),
    };
    let cmd_args = std::env::args().collect::<Vec<String>>();
    let args = Args::parse(&cmd_args, git_dir)
        .unwrap_or_else(|e| exit_with_error("failed to collect args", e.into()));
    debug!("running with {:?}", args);

    let remote_helper = Box::new(
        construct_remote_helper(args)
            .unwrap_or_else(|e| exit_with_error("failed to construct remote helper", e.into())),
    );

    let mut cli = CLI::new(remote_helper, &mut stdin, &mut stdout, &mut stderr);
    cli.run()
        .unwrap_or_else(|e| exit_with_error("failed to run cli", e.into()));
}
