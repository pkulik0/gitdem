pub mod solana;

#[cfg(test)]
pub mod mock;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
}
