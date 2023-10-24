//! _Small library to fetch info about a web page: title, description, language, HTTP info, links, RSS feeds, Opengraph, Schema.org, and more_
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
//! assert_eq!(html.links.len(), 1);
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
//!     headers: Vec<String>,
//! }
//! ```
//!
//! ```rust
//! use webpage::{Webpage, WebpageOptions};
//!
//! let mut options = WebpageOptions::default();
//! options.allow_insecure = true;
//! let info = Webpage::from_url("https://example.org", options).expect("Halp, could not fetch");
//! ```

mod html;
pub use html::{Link, HTML};

#[cfg(feature = "curl")]
mod http;
#[cfg(feature = "curl")]
pub use http::HTTP;

mod opengraph;
pub use opengraph::{Opengraph, OpengraphObject};

mod schema_org;
pub use schema_org::SchemaOrg;

mod parser;

#[cfg(feature = "curl")]
use std::time::Duration;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// All gathered info for a webpage
#[derive(Debug)]
#[cfg(feature = "curl")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Webpage {
    /// info about the HTTP transfer
    pub http: HTTP,
    /// info from the parsed HTML doc
    pub html: HTML,
}

/// Configuration options for fetching a webpage
#[derive(Debug)]
#[cfg(feature = "curl")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct WebpageOptions {
    /// Allow fetching over invalid and/or self signed HTTPS connections \[false\]
    pub allow_insecure: bool,
    /// Follow HTTP redirects \[true\]
    pub follow_location: bool,
    /// Max number of redirects to follow \[5\]
    pub max_redirections: u32,
    /// Timeout for the HTTP request \[10 secs\]
    pub timeout: Duration,
    /// User agent string used for the request \[webpage-rs - <https://crates.io/crates/webpage>\]
    pub useragent: String,
    /// Custom HTTP headers to send with the request
    pub headers: Vec<String>,
}

#[cfg(feature = "curl")]
impl Default for WebpageOptions {
    fn default() -> Self {
        Self {
            allow_insecure: false,
            follow_location: true,
            max_redirections: 5,
            timeout: Duration::from_secs(10),
            useragent: "webpage-rs - https://crates.io/crates/webpage".to_string(),
            headers: Vec::new(),
        }
    }
}

#[cfg(feature = "curl")]
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
    pub fn from_url(url: &str, options: WebpageOptions) -> Result<Self, std::io::Error> {
        let http = HTTP::fetch(url, options)?;

        let html = HTML::from_string(http.body.clone(), Some(http.url.clone()))?;

        Ok(Self { http, html })
    }
}
