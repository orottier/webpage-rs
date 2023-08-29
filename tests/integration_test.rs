extern crate webpage;

#[cfg(feature = "curl")]
use webpage::{html::HTML, Webpage, WebpageOptions};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{header, method};

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

#[tokio::test]
#[cfg(feature = "curl")]
async fn from_url_with_title() {
    let mock_server = MockServer::start().await;
    let response_template = ResponseTemplate::new(200)
        .set_body_string("<html><head><title>testing</title></head><body></body></html>");
    Mock::given(method("GET"))
        .respond_with(response_template)
        .up_to_n_times(1)
        .expect(1)
        .named("from_url_with_title GET")
        .mount(&mock_server)
        .await;

    let webpage = Webpage::from_url(&mock_server.uri(), WebpageOptions::default());
    assert!(webpage.is_ok());

    let html = webpage.unwrap().html;
    assert_eq!(html.title, Some("testing".to_string()));
    assert!(html.description.is_none());
}

#[tokio::test]
#[cfg(feature = "curl")]
async fn from_url_with_headers() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("X-My-Header", "1234"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        // We expect the mock to be called exactly once.
        .expect(1)
        // We assign a name to the mock - it will be shown in error messages
        // if our expectation is not verified!
        .named("from_url_with_headers GET")
        .mount(&mock_server)
        .await;

    let my_headers: Vec<String> = vec!["X-My-Header: 1234".to_string()];
    let options = WebpageOptions { headers: my_headers, ..Default::default()};
    let url = &mock_server.uri();
    let webpage = Webpage::from_url(url, options);
    assert!(webpage.is_ok());
}

#[tokio::test]
#[cfg(feature = "curl")]
async fn from_url_without_headers() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .named("from_url_without_headers GET")
        .mount(&mock_server)
        .await;

    let webpage = Webpage::from_url(&mock_server.uri(), WebpageOptions::default());
    assert!(webpage.is_ok());
}
