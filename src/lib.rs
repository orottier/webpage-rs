extern crate html5ever;
extern crate curl;
extern crate serde_json;

mod http;
pub mod html;
mod opengraph;
mod parser;
mod schema_org;

use http::HTTP;
use html::HTML;

use std::io;
use std::str;

pub struct Webpage {
    pub http: HTTP, // info about the HTTP transfer
    pub html: HTML, // info from the parsed HTML doc
}

impl Webpage {
    pub fn from_url(url: &str) -> Result<Self, io::Error> {
        let http = HTTP::fetch(url)?;

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
