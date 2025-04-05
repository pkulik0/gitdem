use super::LinkOpener;
use std::error::Error;
pub struct BrowserLinkOpener;

impl LinkOpener for BrowserLinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>> {
        open::that(url)?;
        Ok(())
    }
}
