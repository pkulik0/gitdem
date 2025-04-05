use std::error::Error;

use super::browser::LinkOpener;

pub struct MockLinkOpener;

fn send_done_request(url: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/done", url);
    reqwest::blocking::get(url)?
        .text()
        .map(|_| ())
        .map_err(Into::into)
}

impl LinkOpener for MockLinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let url = url.to_string();
        std::thread::spawn(move || match send_done_request(&url) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("mock link opener error: {}", e);
                std::process::exit(1);
            }
        });
        Ok(())
    }
}
