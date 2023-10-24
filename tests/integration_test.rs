extern crate webpage;

use std::io::{Read, Write};
use std::net::TcpListener;

#[cfg(feature = "curl")]
use webpage::{Webpage, WebpageOptions, HTML};

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
fn test_headers() {
    let socket = TcpListener::bind("127.0.0.1:0").unwrap(); // bind to a random port
    let url = format!("{}", socket.local_addr().unwrap());
    std::thread::spawn(move || {
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
    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
}
