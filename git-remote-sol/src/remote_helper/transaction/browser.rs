use rust_embed::RustEmbed;

use super::Executor;
use super::Transaction;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(RustEmbed)]
#[folder = "../wallet-bridge/dist/"]
pub struct BridgeAssets;

pub struct Browser{
    server: tiny_http::Server,
    addr: SocketAddr,
}

impl Browser {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let server = tiny_http::Server::http("localhost:0").map_err(|_| "failed to create server")?;
        let addr = server.server_addr().to_ip().ok_or("failed to get addr")?;
        Ok(Self{server, addr})
    }
}

impl Executor for Browser {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        open::that(format!("http://{}", self.addr))?;
        for request in self.server.incoming_requests() {
            let mut path = request.url().strip_prefix("/").unwrap_or("index.html");
            if path == "" {
                path = "index.html";
            }
            
            let file = BridgeAssets::get(path).ok_or(format!("file not found: {}", path))?;
            let ext = path.split('.').last().ok_or(format!("invalid path: {}", path))?;
            let mime = mime_guess::from_ext(ext).first().ok_or(format!("invalid path: {}", path))?;

            let mut response = tiny_http::Response::from_data(file.data);
            let content_type = tiny_http::Header::from_str(format!("Content-Type: {}", mime).as_str())
                .map_err(|_| format!("failed to create content type header: {}", mime))?;
            response.add_header(content_type);
            request.respond(response)?;
        }
        Ok(())
    }
}
