//! Info about the HTTP transfer

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::time::{Duration, Instant};

use curl::easy::{Easy, List};
use url::{Host, Url};

use crate::WebpageOptions;

fn system_resolve(host: &str, port: u16) -> io::Result<Vec<IpAddr>> {
    Ok((host, port)
        .to_socket_addrs()?
        .map(|address| address.ip())
        .collect())
}

fn pinned_addresses<P, R>(url: &Url, allow: &mut P, resolve: &mut R) -> io::Result<Option<List>>
where
    P: FnMut(IpAddr) -> bool,
    R: FnMut(&str, u16) -> io::Result<Vec<IpAddr>>,
{
    let host = url.host().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "URL must have a destination host",
        )
    })?;
    let port = url.port_or_known_default().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "URL must have a destination port",
        )
    })?;
    let ips = match host {
        Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
        Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
        Host::Domain(name) => resolve(name, port)?,
    };
    if ips.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "destination host has no IP addresses",
        ));
    }
    if !ips.iter().copied().all(allow) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "destination IP address is not public",
        ));
    }
    let Host::Domain(name) = host else {
        return Ok(None);
    };
    let addresses = ips
        .iter()
        .map(|ip| match ip {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => format!("[{ip}]"),
        })
        .collect::<Vec<_>>()
        .join(",");
    let mut list = List::new();
    list.append(&format!("{name}:{port}:{addresses}"))?;
    Ok(Some(list))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return is_public_ipv4(mapped);
            }
            in_ipv6_range(ip, Ipv6Addr::new(0x2000, 0, 0, 0, 0, 0, 0, 0), 3)
                && !in_ipv6_range(ip, Ipv6Addr::new(0x2001, 0, 0, 0, 0, 0, 0, 0), 23)
                && !in_ipv6_range(ip, Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 32)
                && !in_ipv6_range(ip, Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 0), 16)
                && !in_ipv6_range(ip, Ipv6Addr::new(0x3fff, 0, 0, 0, 0, 0, 0, 0), 20)
        }
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    const NON_PUBLIC: &[(u32, u8)] = &[
        (u32::from_be_bytes([0, 0, 0, 0]), 8),
        (u32::from_be_bytes([10, 0, 0, 0]), 8),
        (u32::from_be_bytes([100, 64, 0, 0]), 10),
        (u32::from_be_bytes([127, 0, 0, 0]), 8),
        (u32::from_be_bytes([169, 254, 0, 0]), 16),
        (u32::from_be_bytes([172, 16, 0, 0]), 12),
        (u32::from_be_bytes([192, 0, 0, 0]), 24),
        (u32::from_be_bytes([192, 0, 2, 0]), 24),
        (u32::from_be_bytes([192, 88, 99, 0]), 24),
        (u32::from_be_bytes([192, 168, 0, 0]), 16),
        (u32::from_be_bytes([198, 18, 0, 0]), 15),
        (u32::from_be_bytes([198, 51, 100, 0]), 24),
        (u32::from_be_bytes([203, 0, 113, 0]), 24),
        (u32::from_be_bytes([224, 0, 0, 0]), 4),
        (u32::from_be_bytes([240, 0, 0, 0]), 4),
    ];
    let ip = u32::from(ip);
    !NON_PUBLIC.iter().any(|&(base, prefix)| {
        let mask = u32::MAX << (32 - prefix);
        ip & mask == base & mask
    })
}

fn in_ipv6_range(ip: Ipv6Addr, base: Ipv6Addr, prefix: u8) -> bool {
    let mask = u128::MAX << (128 - prefix);
    u128::from_be_bytes(ip.octets()) & mask == u128::from_be_bytes(base.octets()) & mask
}

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
    /// Fetch a webpage from a public HTTP or HTTPS URL.
    ///
    /// The initial URL and every redirect must use HTTP(S) and must resolve to a public ip address.
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
        Self::fetch_guarded_with(url, options, is_public_ip, system_resolve)
    }

    /// Fetch a URL without scheme or destination-address restrictions.
    ///
    /// This should only be used with trusted URLs. Redirects follow libcurl's defaults.
    ///
    /// ## Examples
    /// ```no_run
    /// use webpage::{HTTP, WebpageOptions};
    ///
    /// let url = "file:///opt/app/pages/trusted.html";
    /// let response = HTTP::fetch_unrestricted(url, WebpageOptions::default())?;
    /// println!("{}", response.body);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn fetch_unrestricted(url: &str, options: WebpageOptions) -> Result<Self, io::Error> {
        Self::fetch_once(url, &options, options.timeout, true, true, None)
    }

    fn fetch_guarded_with<P, R>(
        url: &str,
        options: WebpageOptions,
        mut allow: P,
        mut resolve: R,
    ) -> io::Result<Self>
    where
        P: FnMut(IpAddr) -> bool,
        R: FnMut(&str, u16) -> io::Result<Vec<IpAddr>>,
    {
        let started = Instant::now();
        let mut current =
            Url::parse(url).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
        let initial_origin = current.origin();
        let mut redirects = 0;
        loop {
            if !matches!(current.scheme(), "http" | "https") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "URL scheme must be HTTP or HTTPS",
                ));
            }
            let pinned = pinned_addresses(&current, &mut allow, &mut resolve)?;
            let timeout = if options.timeout.is_zero() {
                Duration::ZERO
            } else {
                options
                    .timeout
                    .checked_sub(started.elapsed())
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::TimedOut, "fetch timeout exceeded")
                    })?
            };
            let forward_headers = current.origin() == initial_origin;
            let mut response = Self::fetch_once(
                current.as_str(),
                &options,
                timeout,
                false,
                forward_headers,
                pinned,
            )?;
            let location = if options.follow_location
                && (300..400).contains(&response.response_code)
            {
                let mut locations = response.headers.iter().filter_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    let value = value.trim();
                    (name.eq_ignore_ascii_case("location") && !value.is_empty()).then_some(value)
                });
                let location = locations.next();
                if locations.next().is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "multiple Location headers in redirect",
                    ));
                }
                location
            } else {
                None
            };
            let Some(location) = location else {
                response.redirect_count = redirects;
                response.transfer_time = started.elapsed();
                return Ok(response);
            };
            if redirects >= options.max_redirections {
                return Err(io::Error::other("maximum redirects exceeded"));
            }
            current = current
                .join(location)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
            redirects += 1;
        }
    }

    fn fetch_once(
        url: &str,
        options: &WebpageOptions,
        timeout: Duration,
        follow_location: bool,
        forward_headers: bool,
        pinned: Option<List>,
    ) -> io::Result<Self> {
        let mut handle = Easy::new();

        // configure
        handle.ssl_verify_peer(!options.allow_insecure)?;
        handle.ssl_verify_host(!options.allow_insecure)?;
        handle.timeout(timeout)?;
        handle.follow_location(follow_location && options.follow_location)?;
        handle.max_redirections(options.max_redirections)?;
        handle.useragent(&options.useragent)?;
        if let Some(pinned) = pinned {
            handle.proxy("")?;
            handle.resolve(pinned)?;
        } else if !follow_location {
            handle.proxy("")?;
        }
        if forward_headers && !options.headers.is_empty() {
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn public_address_policy() {
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "192.0.0.1",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "255.255.255.255",
            "::1",
            "::ffff:127.0.0.1",
            "fc00::1",
            "fe80::1",
            "2001:db8::1",
            "2002:c0a8:0101::1",
            "3fff::1",
        ] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn redirect_is_checked_before_connecting() {
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        let next_url = format!("http://{}", second.local_addr().unwrap());
        let first_url = format!("http://{}", first.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = first.accept().unwrap();
            let mut request = [0; 1024];
            assert!(stream.read(&mut request).unwrap() > 0);
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: {next_url}\r\nContent-Length: 0\r\n\r\n"
            )
            .unwrap();
        });

        let attempts = Rc::new(Cell::new(0));
        let counter = attempts.clone();
        let policy = move |_: IpAddr| {
            let attempt = counter.get();
            counter.set(attempt + 1);
            attempt == 0
        };
        let error = HTTP::fetch_guarded_with(
            &first_url,
            WebpageOptions::default(),
            policy,
            system_resolve,
        )
        .unwrap_err();
        server.join().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(attempts.get(), 2);
        second.set_nonblocking(true).unwrap();
        assert_eq!(
            second.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn guarded_redirect_rejects_ftp() {
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        let next_url = format!("ftp://{}", second.local_addr().unwrap());
        let first_url = format!("http://{}", first.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = first.accept().unwrap();
            let mut request = [0; 1024];
            assert!(stream.read(&mut request).unwrap() > 0);
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: {next_url}\r\nContent-Length: 0\r\n\r\n"
            )
            .unwrap();
        });

        let options = WebpageOptions {
            timeout: Duration::from_secs(2),
            ..WebpageOptions::default()
        };
        let error =
            HTTP::fetch_guarded_with(&first_url, options, |_| true, system_resolve).unwrap_err();
        server.join().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        second.set_nonblocking(true).unwrap();
        assert_eq!(
            second.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn dns_answer_is_pinned_for_the_transfer() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let size = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..size]).contains("Host: pinned.invalid"));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .unwrap();
        });
        let url = format!("http://pinned.invalid:{port}/");
        let response = HTTP::fetch_guarded_with(
            &url,
            WebpageOptions::default(),
            |_| true,
            |host, resolved_port| {
                assert_eq!(host, "pinned.invalid");
                assert_eq!(resolved_port, port);
                Ok(vec![Ipv4Addr::LOCALHOST.into()])
            },
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(response.body, "ok");
        assert_eq!(response.ip, "127.0.0.1");
    }

    #[test]
    fn mixed_dns_answers_are_rejected() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!(
            "http://mixed.invalid:{}/",
            listener.local_addr().unwrap().port()
        );
        let error =
            HTTP::fetch_guarded_with(&url, WebpageOptions::default(), is_public_ip, |_, _| {
                Ok(vec!["8.8.8.8".parse().unwrap(), Ipv4Addr::LOCALHOST.into()])
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        listener.set_nonblocking(true).unwrap();
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn relative_redirect_preserves_final_metadata() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/start", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut first, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            assert!(first.read(&mut request).unwrap() > 0);
            first
                .write_all(b"HTTP/1.1 302 Found\r\nLocation: /done\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
            let (mut second, _) = listener.accept().unwrap();
            assert!(second.read(&mut request).unwrap() > 0);
            second
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .unwrap();
        });
        let response =
            HTTP::fetch_guarded_with(&url, WebpageOptions::default(), |_| true, system_resolve)
                .unwrap();
        server.join().unwrap();
        assert_eq!(response.redirect_count, 1);
        assert!(response.url.ends_with("/done"));
        assert_eq!(response.body, "ok");
        assert_eq!(response.response_code, 200);
    }

    #[test]
    fn cross_origin_redirect_drops_custom_headers() {
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", first.local_addr().unwrap());
        let next_url = format!("http://{}", second.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = first.accept().unwrap();
            let mut request = [0; 1024];
            let size = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..size]).contains("X-Secret: secret"));
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: {next_url}\r\nContent-Length: 0\r\n\r\n"
            )
            .unwrap();
            let (mut stream, _) = second.accept().unwrap();
            let size = stream.read(&mut request).unwrap();
            assert!(!String::from_utf8_lossy(&request[..size]).contains("X-Secret"));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
        });
        let options = WebpageOptions {
            headers: vec!["X-Secret: secret".into()],
            ..WebpageOptions::default()
        };
        HTTP::fetch_guarded_with(&url, options, |_| true, system_resolve).unwrap();
        server.join().unwrap();
    }
}
