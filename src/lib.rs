//! _Small library to fetch info about a web page: title, description, language, HTTP info, RSS feeds, Opengraph, Schema.org, and more_
//!
//! ## Usage
//!
//! ```rust
//! use webpage::{Webpage, WebpageOptions};
//!
//! let info = Webpage::from_url("http://example.org", WebpageOptions::default())
//!     .expect("Could not read from URL");
//!
//! // the HTTP transfer info
//! let http = info.http;
//!
//! // assert_eq!(http.ip, "54.192.129.71".to_string());
//! assert!(http.headers[0].starts_with("HTTP"));
//! assert!(http.body.starts_with("<!doctype html>"));
//! assert_eq!(http.url, "http://example.org/".to_string()); // effective url
//! assert_eq!(http.content_type, "text/html; charset=UTF-8".to_string());
//!
//! // the parsed HTML info
//! let html = info.html;
//!
//! assert_eq!(html.title, Some("Example Domain".to_string()));
//! assert_eq!(html.description, None);
//! assert_eq!(html.opengraph.og_type, "website".to_string());
//! ```
//!
//! You can also get HTML info about local data:
//!
//! ```rust
//! use webpage::HTML;
//! let html = HTML::from_file("index.html", None);
//! // or let html = HTML::from_string(input, None);
//! ```
//!
//! ## Options
//!
//! The following configurations are available:
//! ```rust
//! pub struct WebpageOptions {
//!     allow_insecure: bool,
//!     follow_location: bool,
//!     max_redirections: u32,
//!     timeout: std::time::Duration,
//!     useragent: String,
//! }
//! ```
//!
//! ```
//! use webpage::{Webpage, WebpageOptions};
//!
//! let options = WebpageOptions { allow_insecure: true, ..Default::default() };
//! let info = Webpage::from_url("https://example.org", options).expect("Halp, could not fetch");
//! ```

pub mod html;
pub mod http;
pub mod opengraph;
pub mod schema_org;

mod parser;

pub use crate::html::HTML;
pub use crate::http::HTTP;
pub use crate::opengraph::{Opengraph, OpengraphObject};
pub use crate::schema_org::SchemaOrg;

use std::io;
use std::str;
use std::time::Duration;

/// Resulting info for a webpage
#[derive(Debug)]
pub struct Webpage {
    /// info about the HTTP transfer
    pub http: HTTP,
    /// info from the parsed HTML doc
    pub html: HTML,
}

/// Configuration options
#[derive(Debug)]
pub struct WebpageOptions {
    /// Allow fetching over invalid and/or self signed HTTPS connections \[false\]
    pub allow_insecure: bool,
    /// Follow HTTP redirects \[true\]
    pub follow_location: bool,
    /// Max number of redirects to follow \[5\]
    pub max_redirections: u32,
    /// Timeout for the HTTP request \[10 secs\]
    pub timeout: Duration,
    /// User agent string used for the request \[webpage-rs - https://crates.io/crates/webpage\]
    pub useragent: String,
}

impl Default for WebpageOptions {
    fn default() -> Self {
        Self {
            allow_insecure: false,
            follow_location: true,
            max_redirections: 5,
            timeout: Duration::from_secs(10),
            useragent: "webpage-rs - https://crates.io/crates/webpage".to_string(),
        }
    }
}

impl Webpage {
    /// Fetch a webpage from the given URL, and extract HTML info
    ///
    /// ## Examples
    /// ```
    /// use webpage::{Webpage, WebpageOptions};
    ///
    /// let info = Webpage::from_url("http://example.org", WebpageOptions::default());
    /// assert!(info.is_ok())
    /// ```
    pub fn from_url(url: &str, options: WebpageOptions) -> Result<Self, io::Error> {
        let http = HTTP::fetch(url, options)?;

        let html = HTML::from_string(http.body.clone(), Some(http.url.clone()))?;

        Ok(Self { http, html })
    }
}
