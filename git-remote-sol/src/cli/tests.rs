use crate::cli::CLI;
use crate::remote_helper::mock::Mock;
use crate::remote_helper::reference::{Keyword, Reference, Value};
use std::io::{BufReader, Cursor};

#[test]
fn capabilities() {
    let mut stdin = BufReader::new(Cursor::new(b"capabilities\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let remote_helper = Mock::new();
    let mut cli = CLI::new(Box::new(remote_helper), &mut stdin, &mut stdout, &mut stderr);

    cli.run().expect("failed to run cli");
    assert_eq!(stdout, b"*fetch\n*push\n\n");
    assert_eq!(stderr, b"");
}

#[test]
fn list() {
    // Case 1: No refs
    let mut stdin = BufReader::new(Cursor::new(b"list\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let remote_helper = Mock::new();
    let mut cli = CLI::new(Box::new(remote_helper), &mut stdin, &mut stdout, &mut stderr);
    cli.run().expect("failed to run cli");
    assert_eq!(stdout, b"\n"); // new line indicates the end of the list
    assert_eq!(stderr, b"");

    // Case 2: Some refs
    let mut stdin = BufReader::new(Cursor::new(b"list\n\n".to_vec()));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let refs = vec![
        Reference {
            value: Value::Hash("1234567890".to_string()),
            name: "refs/heads/main".to_string(),
            attributes: vec![],
        },
        Reference {
            value: Value::SymRef("refs/heads/main".to_string()),
            name: "refs/heads/main".to_string(),
            attributes: vec![],
        },
        Reference {
            value: Value::KeyValue(Keyword::ObjectFormat("sha1".to_string())),
            name: "refs/heads/main".to_string(),
            attributes: vec![],
        },
    ];
    let remote_helper = Mock::new_with_refs(refs.clone());
    let mut cli = CLI::new(Box::new(remote_helper), &mut stdin, &mut stdout, &mut stderr);
    cli.run().expect("failed to run cli");
    assert_eq!(stdout, format!("{}\n{}\n{}\n\n", refs[0], refs[1], refs[2]).as_bytes());
    assert_eq!(stderr, b"");
}