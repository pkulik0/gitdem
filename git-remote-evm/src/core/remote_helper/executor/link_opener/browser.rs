use super::LinkOpener;
use crate::core::remote_helper::error::RemoteHelperError;

pub struct BrowserLinkOpener;

impl LinkOpener for BrowserLinkOpener {
    fn open(&self, url: &str) -> Result<(), RemoteHelperError> {
        open::that(url).map_err(|e| RemoteHelperError::Failure {
            action: "opening browser".to_string(),
            details: Some(e.to_string()),
        })?;
        Ok(())
    }
}
