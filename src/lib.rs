extern crate html5ever;
extern crate curl;
extern crate serde_json;

mod http;
pub mod html;
mod opengraph;
mod parser;
mod schema_org;

pub use http::HTTP;
pub use html::HTML;

use std::io;
use std::str;
use std::time::Duration;

pub struct Webpage {
    pub http: HTTP, // info about the HTTP transfer
    pub html: HTML, // info from the parsed HTML doc
}

pub struct WebpageOptions {
    pub allow_insecure: bool,
    pub follow_location: bool,
    pub max_redirections: u32,
    pub timeout: Duration,
    pub useragent: String,
}

impl Default for WebpageOptions {
    fn default() -> Self {
        Self {
            allow_insecure: false,
            follow_location: true,
            max_redirections: 5,
            timeout: Duration::from_secs(10),
            useragent: "Webpage - Rust crate - https://crates.io/crates/webpage".to_string(),
        }
    }
}

impl Webpage {
    pub fn from_url(url: &str, options: WebpageOptions) -> Result<Self, io::Error> {
        let http = HTTP::fetch(url, options)?;

        let html = HTML::from_string(
            http.body.clone(),
            Some(http.url.clone())
        )?;

        Ok(Self {
            http,
            html,
        })
    }
}
