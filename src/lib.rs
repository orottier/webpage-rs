//! _Small library to fetch info about a web page: title, description, language, HTTP info, links, RSS feeds, Opengraph, Schema.org, and more_
//!
//! ## Usage
//!
//! ```no_run
//! use webpage::{Webpage, WebpageOptions};
//!
//! let info = Webpage::from_url("http://example.org", WebpageOptions::default())
//!     .expect("Could not read from URL");
//!
//! // the HTTP transfer info
//! println!("Effective URL: {}", info.http.url);
//! println!("Content type: {}", info.http.content_type);
//!
//! // the parsed HTML info
//! println!("Title: {:?}", info.html.title);
//! println!("Links: {}", info.html.links.len());
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
    /// Fetch a public HTTP or HTTPS webpage, and extract HTML info.
    ///
    /// The initial URL and every redirect must use HTTP(S) and must resolve to a public ip address.
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
        Self::from_http(http)
    }

    /// Fetch without scheme or destination-address restrictions, and extract HTML info.
    ///
    /// This should only be used with trusted URLs. Redirects follow libcurl's defaults.
    ///
    /// ## Examples
    /// ```no_run
    /// use webpage::{Webpage, WebpageOptions};
    ///
    /// let url = "file:///opt/app/pages/trusted.html";
    /// let page = Webpage::from_url_unrestricted(url, WebpageOptions::default())?;
    /// println!("{:?}", page.html.title);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_url_unrestricted(
        url: &str,
        options: WebpageOptions,
    ) -> Result<Self, std::io::Error> {
        let http = HTTP::fetch_unrestricted(url, options)?;
        Self::from_http(http)
    }

    fn from_http(http: HTTP) -> Result<Self, std::io::Error> {
        let html = HTML::from_string(http.body.clone(), Some(http.url.clone()))?;

        Ok(Self { http, html })
    }
}
