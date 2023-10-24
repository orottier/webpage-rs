//! Info from the parsed HTML document

use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;
use url::Url;

use std::collections::HashMap;
use std::default::Default;
use std::io;
use std::path::Path;

use crate::opengraph::Opengraph;
use crate::parser::Parser;
use crate::schema_org::SchemaOrg;

/// Information regarding the HTML content
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct HTML {
    /// \<title\>
    pub title: Option<String>,
    /// meta description
    pub description: Option<String>,
    /// Canonical URL
    pub url: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) url_parsed: Option<Url>,
    /// Feed URL (atom, rss, ..)
    pub feed: Option<String>,

    /// Language as specified in the document
    pub language: Option<String>,
    /// Text content inside \<body\>, all tags stripped
    pub text_content: String,

    /// Flattened down list of meta properties
    pub meta: HashMap<String, String>,
    /// Opengraph tags
    pub opengraph: Opengraph,
    /// Schema.org data
    pub schema_org: Vec<SchemaOrg>,
    /// All links in the document
    pub links: Vec<Link>,
}

impl HTML {
    fn empty(url: Option<String>) -> Self {
        let url_parsed = url.as_ref().and_then(|u| Url::parse(u).ok());
        Self {
            title: None,
            description: None,
            url,
            url_parsed,
            feed: None,

            language: None,
            text_content: String::new(),

            meta: HashMap::new(),
            opengraph: Opengraph::empty(),
            schema_org: Vec::new(),
            links: Vec::new(),
        }
    }

    /// Construct HTML from RcDom, optionally with a URL set
    fn from_dom(dom: RcDom, url: Option<String>) -> Self {
        let mut html = Self::empty(url);
        let parser = Parser::start(dom.document);
        parser.traverse(&mut html);

        html
    }

    /// Construct HTML from File, optionally with a URL set
    pub fn from_file(path: &str, url: Option<String>) -> Result<Self, io::Error> {
        parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .from_file(Path::new(path))
            .map(|dom| Self::from_dom(dom, url))
    }

    /// Construct HTML from String, optionally with a URL set
    ///
    /// ## Examples
    /// ```
    /// use webpage::HTML;
    ///
    /// let input = String::from("<html><head><title>Hello</title></head><body>Contents");
    /// let html = HTML::from_string(input, None);
    /// assert!(html.is_ok());
    ///  ```
    pub fn from_string(html: String, url: Option<String>) -> Result<Self, io::Error> {
        parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map(|dom| Self::from_dom(dom, url))
    }

    pub(crate) fn set_url(&mut self, url: Option<String>) {
        self.url_parsed = url.as_ref().and_then(|url| Url::parse(url).ok());
        self.url = url;
    }
}

/// Information for an `<a>` anchor
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Link {
    pub url: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string() {
        let input = "<html><head><title>Hello</title></head><body>Contents <a href='/a'>Link</a>"
            .to_string();
        let html = HTML::from_string(input, Some("https://example.com/".into()));
        assert!(html.is_ok());

        let html = html.unwrap();
        assert_eq!(html.title, Some("Hello".to_string()));
        assert!(html.description.is_none());
        assert_eq!(html.text_content, "Contents  Link".to_string());
        assert_eq!(
            html.links,
            vec![Link {
                url: "https://example.com/a".into(),
                text: "Link".into()
            }]
        );
    }
}
