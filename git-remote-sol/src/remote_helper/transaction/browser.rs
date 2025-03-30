use mime_guess::Mime;
use rust_embed::RustEmbed;
use std::io::Cursor;

use super::Executor;
use super::Transaction;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use log::trace;

pub trait LinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>>;
}

pub struct BrowserLinkOpener;

impl LinkOpener for BrowserLinkOpener {
    fn open(&self, url: &str) -> Result<(), Box<dyn Error>> {
        open::that(url)?;
        Ok(())
    }
}

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
        let ext = path.split('.').last().ok_or(format!("invalid path: {}", path))?;
        let mime = mime_guess::from_ext(ext).first().ok_or(format!("invalid path: {}", path))?;
        Ok((file.data.to_vec(), mime))
    }
}

pub struct Browser{
    server: tiny_http::Server,
    addr: SocketAddr,
    link_opener: Box<dyn LinkOpener>,
}

impl Browser {
    pub fn new(link_opener: Box<dyn LinkOpener>) -> Result<Self, Box<dyn Error>> {
        let server = tiny_http::Server::http("localhost:0").map_err(|_| "failed to create server")?;
        let addr = server.server_addr().to_ip().ok_or("failed to get addr")?;
        Ok(Self{server, addr, link_opener})
    }
}

fn create_response(data: &[u8], mime: Mime) -> Result<tiny_http::Response<Cursor<Vec<u8>>>, Box<dyn Error>> {
    let mut response = tiny_http::Response::from_data(data);
    let content_type = tiny_http::Header::from_str(format!("Content-Type: {}", mime).as_str())
        .map_err(|_| format!("failed to create content type header: {}", mime))?;
    response.add_header(content_type);
    Ok(response)
}

impl Executor for Browser {
    fn execute(&self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        self.link_opener.open(&format!("http://{}", self.addr))?;
        for request in self.server.incoming_requests() {
            trace!("browser executor request: {:?}", request.url());
            match request.url() {
                "/done" => {
                   request.respond(tiny_http::Response::from_string("done"))?;
                   break;
                },
                "/favicon.ico" => {
                    request.respond(tiny_http::Response::empty(404))?;
                },
                _ => {
                    let (data, mime) = BridgeAssets::from_url(request.url())?;
                    request.respond(create_response(&data, mime)?)?;
                }
            }
        }
        Ok(())
    }
}
