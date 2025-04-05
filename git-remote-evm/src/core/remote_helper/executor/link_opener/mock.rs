use super::LinkOpener;
use crate::core::remote_helper::error::RemoteHelperError;

pub struct MockLinkOpener;

fn send_done_request(url: &str) -> Result<(), RemoteHelperError> {
    let map_err = |e: reqwest::Error| RemoteHelperError::Failure {
        action: "sending done request".to_string(),
        details: Some(e.to_string()),
    };

    let url = format!("{}/done", url);
    reqwest::blocking::get(url)
        .map_err(map_err)?
        .text()
        .map(|_| ())
        .map_err(map_err)
}

impl LinkOpener for MockLinkOpener {
    fn open(&self, url: &str) -> Result<(), RemoteHelperError> {
        let url = url.to_string();
        std::thread::spawn(move || match send_done_request(&url) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("mock link opener error: {}", e);
                std::process::exit(1);
            }
        });
        Ok(())
    }
}
