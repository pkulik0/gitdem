use std::error::Error;

pub mod browser;
#[cfg(test)]
pub mod mock;

pub trait LinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>>;
}
