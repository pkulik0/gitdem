use crate::core::remote_helper::error::RemoteHelperError;

pub mod browser;
#[cfg(test)]
pub mod mock;

pub trait LinkOpener {
    fn open(&self, url: &str) -> Result<(), RemoteHelperError>;
}
