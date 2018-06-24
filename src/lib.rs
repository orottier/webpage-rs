extern crate html5ever;
extern crate curl;

mod http;
mod html;
mod opengraph;

use http::HTTP;
use html::HTML;

use std::str;

pub struct Webpage {
    pub http: Option<HTTP>,
    pub html: Option<HTML>,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
