use mime_guess::Mime;
use rust_embed::RustEmbed;
use std::error::Error;

#[derive(RustEmbed)]
#[folder = "../wallet-bridge/dist/"]
pub struct BridgeAssets;

impl BridgeAssets {
    pub fn from_url(url: &str) -> Result<(Vec<u8>, Mime), Box<dyn Error>> {
        let mut path = url.strip_prefix("/").unwrap_or("index.html");
        if path == "" {
            path = "index.html";
        }

        let file = BridgeAssets::get(path).ok_or(format!("file not found: {}", path))?;
        let ext = path
            .split('.')
            .last()
            .ok_or(format!("invalid path: {}", path))?;
        let mime = mime_guess::from_ext(ext)
            .first()
            .ok_or(format!("invalid path: {}", path))?;
        Ok((file.data.to_vec(), mime))
    }
}

#[test]
fn test_bridge_assets() {
    assert!(BridgeAssets::iter().count() > 0);
    assert!(BridgeAssets::get("index.html").is_some());
}
