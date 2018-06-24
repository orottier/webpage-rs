extern crate webpage;

use webpage::Webpage;

#[test]
fn from_string() {
    let input = "<html><head><title>Hello</title>";
    let webpage = Webpage::from_string(input);
    assert!(webpage.html.is_some());

    let html = webpage.html.unwrap();
    assert_eq!(html.title, Some("Hello".to_string()));
    assert!(html.description.is_none());
}

#[test]
fn from_file() {
    let path = "tests/data/index.html";
    let webpage = Webpage::from_file(path);
    assert!(webpage.html.is_some());

    let html = webpage.html.unwrap();
    assert_eq!(html.title, Some("Example Domain".to_string()));
    assert!(html.description.is_none());
}

#[test]
#[ignore]
fn from_url() {
    let url = "https://example.org";
    let webpage = Webpage::from_url(url);
    assert!(webpage.html.is_some());

    let html = webpage.html.unwrap();
    assert_eq!(html.title, Some("Example Domain".to_string()));
    assert!(html.description.is_none());
}
