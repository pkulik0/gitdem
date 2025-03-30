use crate::remote_helper::mock::Mock;

use super::browser::{BridgeAssets, Browser, BrowserLinkOpener, LinkOpener};
use super::{Executor, Transaction};
use std::error::Error;
use std::thread;

#[test]
fn test_bridge_assets() {
    assert!(BridgeAssets::iter().count() > 0);
    assert!(BridgeAssets::get("index.html").is_some());
}

struct MockLinkOpener;

fn send_done_request(url: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/done", url);
    reqwest::blocking::get(url)?.text().map(|_| ()).map_err(Into::into)
}

impl LinkOpener for MockLinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let url = url.to_string();
        std::thread::spawn(move || {
            match send_done_request(&url) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("mock link opener error: {}", e);
                    std::process::exit(1);
                }
            }
        });
        Ok(())
    }
}

#[test]
fn test_browser() {
    let browser = Browser::new(Box::new(MockLinkOpener)).expect("failed to create browser");
    browser.execute(Transaction).expect("failed to execute transaction");
}
