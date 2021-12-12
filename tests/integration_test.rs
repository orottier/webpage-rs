extern crate webpage;

#[cfg(feature = "curl")]
use webpage::{html::HTML, Webpage, WebpageOptions};

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
