mod cli;
mod config;
mod remote_helper;

use remote_helper::solana::Solana;
use config::git::GitConfig;
use cli::CLI;

use std::io;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let config = Box::new(GitConfig::new());
    let remote_helper = Box::new(Solana::new(config));
    let mut cli = CLI::new(remote_helper, &mut stdin, &mut stdout, &mut stderr);

    cli.run()
}
