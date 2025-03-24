use super::browser::{BridgeAssets, Browser};
use super::{Executor, Transaction};

#[test]
fn test_bridge_assets() {
    assert!(BridgeAssets::iter().count() > 0);
    assert!(BridgeAssets::get("index.html").is_some());
}

#[test]
fn test_browser() {
    let browser = Browser::new().unwrap();
    browser.execute(Transaction{}).unwrap();
}
