use std::error::Error;

use reference::Reference;

pub mod solana;
pub mod reference;
mod transaction;

#[cfg(any(test, feature = "mock"))]
pub mod mock;
#[cfg(test)]
pub mod tests;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
    fn list(&self) -> Result<Vec<Reference>, Box<dyn Error>>;
}

pub trait Wallet {
    fn is_extension(&self) -> bool;
}
