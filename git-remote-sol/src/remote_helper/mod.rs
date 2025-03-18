pub mod solana;
pub mod mock;

pub trait RemoteHelper {
    fn capabilities(&self) -> Vec<&'static str>;
}
