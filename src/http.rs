//! Info about the HTTP transfer

use std::io;
use std::time::Duration;

use curl::easy::{Easy, List};

use crate::WebpageOptions;

/// Information regarding the HTTP transfer
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct HTTP {
    /// The external ip address (v4 or v6)
    pub ip: String,
    /// Duration of the HTTP call
    pub transfer_time: Duration,
    /// Number of redirections encountered
    pub redirect_count: u32,
    /// HTTP content type returned
    pub content_type: String,
    /// HTTP response code returned
    pub response_code: u32,
    /// All HTTP response headers
    pub headers: Vec<String>,
    /// Effective URL that was visited
    pub url: String,
    /// HTTP body
    pub body: String,
}

impl HTTP {
    /// Fetch a webpage from the given URL
    ///
    /// ## Examples
    /// ```
    /// use webpage::HTTP;
    /// use webpage::WebpageOptions;
    ///
    /// let info = HTTP::fetch("http://example.org", WebpageOptions::default());
    /// assert!(info.is_ok());
    ///
    /// let info = HTTP::fetch("mal formed or unreachable", WebpageOptions::default());
    /// assert!(info.is_err());
    /// ```
    pub fn fetch(url: &str, options: WebpageOptions) -> Result<Self, io::Error> {
        let mut handle = Easy::new();

        // configure
        handle.ssl_verify_peer(!options.allow_insecure)?;
        handle.ssl_verify_host(!options.allow_insecure)?;
        handle.timeout(options.timeout)?;
        handle.follow_location(options.follow_location)?;
        handle.max_redirections(options.max_redirections)?;
        handle.useragent(&options.useragent)?;
        if !options.headers.is_empty() {
            let mut list = List::new();
            for header in options.headers.iter() {
                list.append(header)?;
            }
            handle.http_headers(list)?;
        }

        handle.url(url)?;

        let mut headers = Vec::new();
        let mut body = Vec::new();
        {
            let mut transfer = handle.transfer();
            transfer.header_function(|new_data| {
                let header = String::from_utf8_lossy(new_data)
                    .into_owned()
                    .trim()
                    .to_string();

                // clear list on redirects
                if header.starts_with("HTTP/") {
                    headers = Vec::new();
                }

                if !header.is_empty() {
                    headers.push(header);
                }

                true
            })?;

            transfer.write_function(|new_data| {
                body.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;

            transfer.perform()?;
        }

        let body = String::from_utf8_lossy(&body).into_owned();

        Ok(HTTP {
            ip: handle.primary_ip()?.unwrap_or("").to_string(),
            transfer_time: handle.total_time()?,
            redirect_count: handle.redirect_count()?,
            content_type: handle.content_type()?.unwrap_or("").to_string(),
            response_code: handle.response_code()?,
            url: handle.effective_url()?.unwrap_or("").to_string(),

            headers,
            body,
        })
    }
}
