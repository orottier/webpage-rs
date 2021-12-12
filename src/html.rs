//! Info from the parsed HTML document

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;

use std::collections::HashMap;
use std::default::Default;
use std::io;
use std::path::Path;

use html5ever::driver::ParseOpts;
use markup5ever_rcdom::RcDom;

use crate::opengraph::Opengraph;
use crate::parser::Parser;
use crate::schema_org::SchemaOrg;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HTML {
    /// \<title\>
    pub title: Option<String>,
    /// meta description
    pub description: Option<String>,
    /// Canonical URL
    pub url: Option<String>,
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
}

impl HTML {
    fn empty(url: Option<String>) -> Self {
        Self {
            title: None,
            description: None,
            url,
            feed: None,

            language: None,
            text_content: String::new(),

            meta: HashMap::new(),
            opengraph: Opengraph::empty(),
            schema_org: Vec::new(),
        }
    }

    /// Construct HTML from RcDom, optionally with a URL set
    pub fn from_dom(dom: RcDom, url: Option<String>) -> Self {
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
            .and_then(|dom| Ok(Self::from_dom(dom, url)))
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
            .and_then(|dom| Ok(Self::from_dom(dom, url)))
    }
}

#[cfg(test)]
mod tests {
    use super::HTML;

    #[test]
    fn from_string() {
        let input = "<html><head><title>Hello</title></head><body>Contents".to_string();
        let html = HTML::from_string(input, None);
        assert!(html.is_ok());

        let html = html.unwrap();
        assert_eq!(html.title, Some("Hello".to_string()));
        assert!(html.description.is_none());
        assert_eq!(html.text_content, "Contents".to_string());
    }
}
