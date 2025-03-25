mod cli;
mod config;
mod remote_helper;

use log::info;
use remote_helper::solana::Solana;
use config::git::GitConfig;
use cli::CLI;

use std::io;
use std::error::Error;
use flexi_logger::{FileSpec, Logger, WriteMode};

fn main() -> Result<(), Box<dyn Error>> {
    let _logger = Logger::try_with_str("trace")?
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::BufferAndFlush)
        .start()?;

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let remote_name = std::env::args().nth(1).unwrap_or_default();  
    let remote_url = std::env::args().nth(2).unwrap_or_default();

    let config = Box::new(GitConfig::new());
    let remote_helper = Box::new(Solana::new(config));
    let mut cli = CLI::new(remote_helper, &mut stdin, &mut stdout, &mut stderr, remote_name, remote_url);

    cli.run()
}
