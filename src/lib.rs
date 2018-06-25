extern crate html5ever;
extern crate curl;

mod http;
mod html;
mod opengraph;
mod parser;

use http::HTTP;
use html::HTML;

use std::str;

pub struct Webpage {
    pub http: Option<HTTP>, // info about the HTTP transfer, if any
    pub html: Option<HTML>, // info from the parsed HTML doc, if any
}

impl Webpage {
    pub fn from_file(path: &str) -> Self {
        let html = HTML::from_file(path);

        Self {
            http: None,
            html,
        }
    }

    pub fn from_string(body: &str) -> Self {
        let html = HTML::from_string(body.to_string(), None);

        Self {
            http: None,
            html,
        }
    }

    pub fn from_url(url: &str) -> Self {
        let http = HTTP::fetch(url);

        let html = HTML::from_string(
            http.body.clone(),
            Some(http.url.clone())
        );

        Self {
            http: Some(http),
            html,
        }
    }
}
