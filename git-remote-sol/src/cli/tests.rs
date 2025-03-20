use crate::cli::CLI;
use crate::remote_helper::mock::Mock;
use std::io::{BufReader, Cursor};

#[test]
fn capabilities() {
    let mut stdin = BufReader::new(Cursor::new(b"capabilities\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let remote_helper = Mock::new();
    let mut cli = CLI::new(Box::new(remote_helper), &mut stdin, &mut stdout, &mut stderr);

    cli.run().expect("failed to run cli");
    assert_eq!(stdout, b"fetch,push\n");
    assert_eq!(stderr, b"");
}
