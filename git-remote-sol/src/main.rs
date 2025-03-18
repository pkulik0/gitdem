mod cli;
mod remote_helper;

use remote_helper::solana::Solana;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let remote_helper = Solana::new();
    let mut cli = cli::CLI::new(&remote_helper, &mut stdin, &mut stdout, &mut stderr);
    
    cli.run()
}
