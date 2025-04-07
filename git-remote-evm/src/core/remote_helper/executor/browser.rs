// use log::trace;
// use mime_guess::Mime;
// use rust_embed::RustEmbed;
// use std::error::Error;
// use std::io::Cursor;
// use std::net::SocketAddr;
// use std::str::FromStr;

// use super::Executor;
// use super::Transaction;
// use super::link_opener::LinkOpener;
// use crate::core::remote_helper::error::RemoteHelperError;
// #[derive(RustEmbed)]
// #[folder = "../wallet-bridge/dist/"]
// pub struct BridgeAssets;

// impl BridgeAssets {
//     pub fn from_url(url: &str) -> Result<(Vec<u8>, Mime), RemoteHelperError> {
//         let mut path = url.strip_prefix("/").unwrap_or("index.html");
//         if path == "" {
//             path = "index.html";
//         }

//         let file = BridgeAssets::get(path).ok_or(RemoteHelperError::Failure {
//             action: "getting asset".to_string(),
//             details: Some(format!("file not found: {}", path)),
//         })?;
//         let ext = path.split('.').last().ok_or(RemoteHelperError::Failure {
//             action: "getting asset extension".to_string(),
//             details: Some(format!("invalid path: {}", path)),
//         })?;
//         let mime = mime_guess::from_ext(ext)
//             .first()
//             .ok_or(RemoteHelperError::Failure {
//                 action: "getting asset mime type".to_string(),
//                 details: Some(format!("invalid path: {}", path)),
//             })?;
//         Ok((file.data.to_vec(), mime))
//     }
// }

// #[test]
// fn test_bridge_assets() {
//     assert!(BridgeAssets::iter().count() > 0);
//     assert!(BridgeAssets::get("index.html").is_some());
// }

// pub struct Browser {
//     server: tiny_http::Server,
//     addr: SocketAddr,
//     link_opener: Box<dyn LinkOpener>,
// }

// impl Browser {
//     pub fn new(link_opener: Box<dyn LinkOpener>) -> Result<Self, RemoteHelperError> {
//         let server =
//             tiny_http::Server::http("localhost:0").map_err(|e| RemoteHelperError::Failure {
//                 action: "creating browser executor".to_string(),
//                 details: Some(e.to_string()),
//             })?;
//         let addr = server
//             .server_addr()
//             .to_ip()
//             .ok_or(RemoteHelperError::Failure {
//                 action: "getting local server's address".to_string(),
//                 details: None,
//             })?;
//         Ok(Self {
//             server,
//             addr,
//             link_opener,
//         })
//     }
// }

// fn create_response(data: &[u8], mime: Mime) -> Option<tiny_http::Response<Cursor<Vec<u8>>>> {
//     let mut response = tiny_http::Response::from_data(data);
//     let content_type =
//         tiny_http::Header::from_str(format!("Content-Type: {}", mime).as_str()).ok()?;
//     response.add_header(content_type);
//     Some(response)
// }

// impl Executor for Browser {
//     fn execute(&self, transaction: Transaction) -> Result<(), RemoteHelperError> {
//         self.link_opener.open(&format!("http://{}", self.addr))?;
//         for request in self.server.incoming_requests() {
//             trace!("browser executor request: {:?}", request.url());
//             match request.url() {
//                 "/done" => {
//                     request
//                         .respond(tiny_http::Response::from_string("done"))
//                         .map_err(|e| RemoteHelperError::Failure {
//                             action: "sending executor response".to_string(),
//                             details: Some(e.to_string()),
//                         })?;
//                     break;
//                 }
//                 "/favicon.ico" => {
//                     request
//                         .respond(tiny_http::Response::empty(404))
//                         .map_err(|e| RemoteHelperError::Failure {
//                             action: "sending executor response".to_string(),
//                             details: Some(e.to_string()),
//                         })?;
//                 }
//                 _ => {
//                     let (data, mime) = BridgeAssets::from_url(request.url())?;
//                     let response = create_response(&data, mime).ok_or(
//                         RemoteHelperError::Failure {
//                             action: "creating response".to_string(),
//                             details: None,
//                         },
//                     )?;
//                     request.respond(response).map_err(|e| RemoteHelperError::Failure {
//                         action: "sending executor response".to_string(),
//                         details: Some(e.to_string()),
//                     })?;
//                 }
//             }
//         }
//         Ok(())
//     }
// }

// #[test]
// fn test_browser() {
//     use super::link_opener::mock::MockLinkOpener;
//     let browser = Browser::new(Box::new(MockLinkOpener)).expect("failed to create browser");
//     browser
//         .execute(Transaction)
//         .expect("failed to execute transaction");
// }
