#![feature(slice_as_array)]

mod args;
mod cli;
mod core;
#[cfg(test)]
mod e2e_tests;
mod macros;

use args::Args;
use cli::CLI;
use core::git::Git;
use core::kv_source::EnvSource;
use core::remote_helper::executor::Background;
use core::remote_helper::{error::RemoteHelperError, evm::Evm};
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{debug, error, warn};
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

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
    use core::remote_helper::config::Config;

    debug!("using evm remote helper");
    let git = Rc::new(SystemGit::new(args.directory().clone()));

    let git_version = git.version()?;
    debug!("git version: {}", git_version);
    if git_version.major < 3 && git_version.minor < 42 {
        warn!("sha256 has been fully supported since git 2.42.0, unexpected results may occur");
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| RemoteHelperError::Failure {
            action: "creating runtime".to_string(),
            details: Some(e.to_string()),
        })?;
    let env_source = Rc::new(EnvSource::new());
    let config = Config::new(args.protocol().to_string(), vec![env_source, git.clone()]);

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

    let executor = runtime.block_on(Background::new(
        config.get_wallet()?,
        &config.get_rpc()?,
        address,
    ))?;

    Evm::new(runtime, Box::new(executor), git)
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

    let git_dir_var = std::env::var(GIT_DIR_ENV_VAR).unwrap_or_else(|e| {
        exit_with_error("failed to get git dir", e.into());
    });
    let git_dir = PathBuf::from(git_dir_var);

    let cmd_args = std::env::args().collect::<Vec<String>>();
    let args = Args::parse(&cmd_args, git_dir)
        .unwrap_or_else(|e| exit_with_error("failed to collect args", e.into()));
    debug!("running with {:?}", args);

    let remote_helper = construct_remote_helper(args)
        .unwrap_or_else(|e| exit_with_error("failed to construct remote helper", e.into()));

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();

    let mut cli = CLI::new(Box::new(remote_helper), &mut stdin, &mut stdout);
    cli.run()
        .unwrap_or_else(|e| exit_with_error("failed to run cli", e.into()));
}
