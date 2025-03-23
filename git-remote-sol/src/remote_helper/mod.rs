pub mod solana;
mod transaction;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
}
