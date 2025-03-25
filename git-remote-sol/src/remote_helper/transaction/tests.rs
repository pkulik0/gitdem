use super::browser::{BridgeAssets, Browser, BrowserLinkOpener, LinkOpener};
use super::{Executor, Transaction};
use std::error::Error;

#[test]
fn test_bridge_assets() {
    assert!(BridgeAssets::iter().count() > 0);
    assert!(BridgeAssets::get("index.html").is_some());
}

struct MockLinkOpener;

impl LinkOpener for MockLinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[test]
fn test_browser() {
    let browser = Browser::new(Box::new(BrowserLinkOpener)).expect("failed to create browser");
    browser.execute(Transaction).expect("failed to execute transaction");
}
