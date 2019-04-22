use crate::WebpageOptions;

use curl::easy::Easy;
use std::io;
use std::time::Duration;

#[derive(Debug)]
pub struct HTTP {
    pub ip: String,
    pub transfer_time: Duration,
    pub redirect_count: u32,
    pub content_type: String,
    pub response_code: u32,
    pub headers: Vec<String>,
    pub url: String, // effective url
    pub body: String,
}

impl HTTP {
    pub fn fetch(url: &str, options: WebpageOptions) -> Result<Self, io::Error> {
        let mut handle = Easy::new();

        // configure
        handle.ssl_verify_peer(!options.allow_insecure)?;
        handle.ssl_verify_host(!options.allow_insecure)?;
        handle.timeout(options.timeout)?;
        handle.follow_location(options.follow_location)?;
        handle.max_redirections(options.max_redirections)?;
        handle.useragent(&options.useragent)?;

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
