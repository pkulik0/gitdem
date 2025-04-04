mod args;
mod cli;
mod config;
mod remote_helper;
#[cfg(test)]
mod integration_tests;

use args::Args;
use cli::CLI;
#[cfg(feature = "mock")]
use remote_helper::mock::Mock;
#[cfg(not(feature = "mock"))]
use remote_helper::evm::helper::Evm;

use flexi_logger::{FileSpec, Logger, WriteMode};
use log::{debug, error, warn};
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

#[cfg(not(feature = "mock"))]
fn construct_remote_helper(args: Args) -> Evm {
    use config::git::GitConfig;

    debug!("using evm remote helper");
    let config = Box::new(GitConfig::new(args.directory().clone()));
    Evm::new(args, config)
}

#[cfg(feature = "mock")]
fn construct_remote_helper(_: Args) -> Mock {
    use config::mock::MockConfig;
    use remote_helper::hash::Hash;
    use remote_helper::reference::{Keyword, Reference, Value};

    warn!("using mock remote helper");
    Mock::new(vec![
        // Reference {
        //     value: Value::KeyValue(Keyword::ObjectFormat("sha1".to_string())),
        //     name: "".to_string(),
        //     attributes: vec![],
        // },
        // Reference {
        //     value: Value::Hash(
        //         Hash::from_str("4e1243bd22c66e76c2ba9eddc1f91394e57f9f83").expect("invalid hash"),
        //     ),
        //     name: "refs/heads/main".to_string(),
        //     attributes: vec![],
        // },
        // Reference {
        //     value: Value::SymRef("refs/heads/main".to_string()),
        //     name: "HEAD".to_string(),
        //     attributes: vec![],
        // },
    ])
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

    let remote_helper = Box::new(construct_remote_helper(args));

    let mut cli = CLI::new(remote_helper, &mut stdin, &mut stdout, &mut stderr);
    cli.run()
        .unwrap_or_else(|e| exit_with_error("failed to run cli", e.into()));
}
