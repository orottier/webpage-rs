# Webpage.rs

_Get some info about a webpage_

## Usage

```rust
extern crate webpage;
use webpage::Webpage;

// ...

info = Webpage::from_url("http://www.rust-lang.org/en-US/");

// the HTTP info
let http = info.http.unwrap();

assert_eq!(http.ip, "54.192.129.71".to_string());
assert!(http.headers[0].starts_with("HTTP"));
assert!(http.body.starts_with("<!DOCTYPE html>"));
assert_eq!(http.url, "https://www.rust-lang.org/en-US/".to_string()); // followed redirects
assert_eq!(http.content_type, "text/html".to_string());

// the HTML info
let html = info.html.unwrap();

assert_eq!(html.title.unwrap(), "The Rust Programming Language".to_string());
assert_eq!(html.description.unwrap(), "A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string());
assert_eq!(html.opengraph.og_type, "website".to_string());
