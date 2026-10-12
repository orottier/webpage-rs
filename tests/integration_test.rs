extern crate webpage;

#[cfg(feature = "curl")]
use std::io::{Read, Write};
#[cfg(feature = "curl")]
use std::net::TcpListener;

use webpage::HTML;
#[cfg(feature = "curl")]
use webpage::{Webpage, WebpageOptions, HTTP};

#[test]
fn from_file() {
    let path = "tests/data/index.html";
    let html = HTML::from_file(path, None);
    assert!(html.is_ok());

    let html = html.unwrap();
    assert_eq!(html.title, Some("Example Domain".to_string()));
    assert!(html.description.is_none());
}

#[test]
#[ignore]
#[cfg(feature = "curl")]
fn from_url() {
    let url = "https://example.org";
    let webpage = Webpage::from_url(url, WebpageOptions::default());
    assert!(webpage.is_ok());

    let html = webpage.unwrap().html;
    assert_eq!(html.title, Some("Example Domain".to_string()));
    assert!(html.description.is_none());
}

#[test]
#[cfg(feature = "curl")]
fn non_http_urls_are_rejected() {
    for url in [
        "file:///tmp/index.html",
        "ftp://example.org/index.html",
        "127.0.0.1:8080",
    ] {
        let error = HTTP::fetch(url, WebpageOptions::default()).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);

        let error = Webpage::from_url(url, WebpageOptions::default()).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }
}

#[test]
#[cfg(feature = "curl")]
fn test_headers() {
    let socket = TcpListener::bind("127.0.0.1:0").unwrap(); // bind to a random port
    let url = format!("http://{}", socket.local_addr().unwrap());
    let task = std::thread::spawn(move || {
        let my_headers: Vec<String> = vec!["X-My-Header: 1234".to_string()];
        let mut options = WebpageOptions::default();
        options.headers = my_headers;
        let webpage = Webpage::from_url(&url, options);
        assert!(webpage.is_ok());
    });
    let mut stream = socket.accept().unwrap().0;
    let mut buf = vec![0; 1024];
    let mut read = 0;
    let mut request;
    loop {
        let bytes = stream.read(&mut buf[read..]).unwrap();
        assert_ne!(bytes, 0);
        read += bytes;
        request = String::from_utf8(buf[..read].to_vec()).unwrap();
        if request.contains("\r\n\r\n") {
            break;
        }
    }
    assert!(request.contains("X-My-Header: 1234\r\n"));
    stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .unwrap();
    task.join().unwrap();
}

#[test]
#[cfg(feature = "curl")]
fn from_url_any_scheme_accepts_scheme_less_url() {
    let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = socket.local_addr().unwrap().to_string();
    let task =
        std::thread::spawn(move || Webpage::from_url_any_scheme(&url, WebpageOptions::default()));

    let (mut stream, _) = socket.accept().unwrap();
    let mut request = [0; 1024];
    let size = stream.read(&mut request).unwrap();
    assert!(std::str::from_utf8(&request[..size])
        .unwrap()
        .starts_with("GET "));
    stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .unwrap();

    let webpage = task.join().unwrap().unwrap();
    assert_eq!(webpage.http.response_code, 200);
}
