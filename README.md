# Webpage.rs

_Small library to fetch info about a web page: title, description, language, HTTP info, RSS feeds, Opengraph, Schema.org, and more_

## Usage

```rust
use webpage::{Webpage, WebpageOptions};

let info = Webpage::from_url("http://www.rust-lang.org/en-US/", WebpageOptions::default())
    .expect("Could not read from URL");

// the HTTP transfer info
let http = info.http;

assert_eq!(http.ip, "54.192.129.71".to_string());
assert!(http.headers[0].starts_with("HTTP"));
assert!(http.body.starts_with("<!DOCTYPE html>"));
assert_eq!(http.url, "https://www.rust-lang.org/en-US/".to_string()); // followed redirects (HTTPS)
assert_eq!(http.content_type, "text/html".to_string());

// the parsed HTML info
let html = info.html;

assert_eq!(html.title, Some("The Rust Programming Language".to_string()));
assert_eq!(html.description, Some("A systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.".to_string()));
assert_eq!(html.opengraph.og_type, "website".to_string());
```

You can also get HTML info about local data:

```rust
use webpage::HTML;
let html = HTML::from_file("index.html", None);
// or let html = HTML::from_string(input, None);
```

## All fields

```rust
pub struct Webpage {
    pub http: HTTP, // info about the HTTP transfer
    pub html: HTML, // info from the parsed HTML doc
}

pub struct HTTP {
    pub ip: String,
    pub transfer_time: Duration,
    pub redirect_count: u32,
    pub content_type: String,
    pub response_code: u32,
    pub headers: Vec<String>, // raw headers from final request
    pub url: String, // effective url
    pub body: String,
}

pub struct HTML {
    pub title: Option<String>,
    pub description: Option<String>,

    pub url: Option<String>, // canonical url
    pub feed: Option<String>, // RSS feed typically

    pub language: Option<String>, // as specified, not detected
    pub text_content: String, // all tags stripped from body

    pub meta: HashMap<String, String>, // flattened down list of meta properties

    pub opengraph: Opengraph,
    pub schema_org: Vec<SchemaOrg>,
}

pub struct Opengraph {
    pub og_type: String,
    pub properties: HashMap<String, String>,

    pub images: Vec<Object>,
    pub videos: Vec<Object>,
    pub audios: Vec<Object>,
}

// Facebook's Opengraph structured data
pub struct OpengraphObject {
    pub url: String,
    pub properties: HashMap<String, String>,
}

// Google's schema.org structured data
pub struct SchemaOrg {
    pub schema_type: String,
    pub value: serde_json::Value,
}
```

## Options

The following configurations are available:
```rust
pub struct WebpageOptions {
    allow_insecure: false,
    follow_location: true,
    max_redirections: 5,
    timeout: Duration::from_secs(10),
    useragent: "Webpage - Rust crate - https://crates.io/crates/webpage".to_string(),
}

// usage
let options = WebpageOptions { allow_insecure: true, ..Default::default() };
let info = Webpage::from_url(&url, options).expect("Halp, could not fetch");
```
